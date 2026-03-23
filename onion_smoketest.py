#!/usr/bin/env python3
"""
Onion-mode smoke test for PP2P protocol.

This validates:
1) two local nodes can discover each other via Tor onion rendezvous,
2) they can exchange chat,
3) forced drop auto-reconnects.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import shutil
import time
from pathlib import Path

from pp2p import Contact, PP2PNode, Rendezvous, RuntimeConfig, save_contacts, state_identity


async def wait_until(pred, timeout: float, label: str) -> None:
    deadline = time.monotonic() + timeout
    while not pred():
        if time.monotonic() > deadline:
            raise TimeoutError(f"Timed out waiting for: {label}")
        await asyncio.sleep(0.5)


def make_cfg(
    state_dir: Path,
    signal_port: int,
    socks_port: int,
    control_port: int,
    tor_bin: str | None,
) -> RuntimeConfig:
    return RuntimeConfig(
        state_dir=state_dir,
        mode="onion",
        signal_host="127.0.0.1",
        signal_port=signal_port,
        advertise_host="127.0.0.1",
        retry_seconds=5.0,
        stun_server="stun:stun.l.google.com:19302",
        turn_server=None,
        turn_username=None,
        turn_password=None,
        turn_secret=None,
        turn_ttl_seconds=3600,
        turn_user="pp2p",
        tor_bin=tor_bin,
        tor_socks_port=socks_port,
        tor_control_port=control_port,
        no_stdin=True,
    )


async def run_test(base_dir: Path, tor_bin: str | None) -> None:
    if base_dir.exists():
        shutil.rmtree(base_dir)
    (base_dir / "alice").mkdir(parents=True, exist_ok=True)
    (base_dir / "bob").mkdir(parents=True, exist_ok=True)

    alice_state = base_dir / "alice"
    bob_state = base_dir / "bob"

    alice_id = state_identity(alice_state)
    bob_id = state_identity(bob_state)

    alice = PP2PNode(make_cfg(alice_state, signal_port=18201, socks_port=19250, control_port=19251, tor_bin=tor_bin))
    bob = PP2PNode(make_cfg(bob_state, signal_port=18202, socks_port=19350, control_port=19351, tor_bin=tor_bin))

    alice_task = asyncio.create_task(alice.run(), name="alice-onion")
    bob_task = asyncio.create_task(bob.run(), name="bob-onion")

    try:
        await wait_until(
            lambda: bool(alice._own_rendezvous is not None and bob._own_rendezvous is not None),
            timeout=180,
            label="onion rendezvous bootstrapped",
        )
        alice_onion = alice.own_rendezvous.address
        bob_onion = bob.own_rendezvous.address
        print(f"Alice onion: {alice_onion}")
        print(f"Bob onion:   {bob_onion}")

        # Build and persist trusted contact sets while nodes are running.
        alice_contact = Contact(
            peer_id=bob_id.peer_id,
            public_key_b64=bob_id.public_key_b64,
            rendezvous=Rendezvous(transport="onion", address=bob_onion, port=80),
            name="bob",
        )
        bob_contact = Contact(
            peer_id=alice_id.peer_id,
            public_key_b64=alice_id.public_key_b64,
            rendezvous=Rendezvous(transport="onion", address=alice_onion, port=80),
            name="alice",
        )
        save_contacts(alice_state, {bob_id.peer_id: alice_contact})
        save_contacts(bob_state, {alice_id.peer_id: bob_contact})

        # Refresh each running node's contact/session map and start dial loops.
        alice._load_contacts()
        bob._load_contacts()
        alice._start_maintainers()
        bob._start_maintainers()

        await wait_until(
            lambda: bool(alice.sessions.get(bob_id.peer_id) and bob.sessions.get(alice_id.peer_id)),
            timeout=30,
            label="session maps initialized",
        )
        await wait_until(
            lambda: bool(alice.sessions[bob_id.peer_id].connected and bob.sessions[alice_id.peer_id].connected),
            timeout=240,
            label="initial onion connection",
        )
        print("Initial onion connection established")

        await alice.send_chat(bob_id.peer_id, "hello-over-onion-signal")
        await bob.send_chat(alice_id.peer_id, "hello-back")
        await asyncio.sleep(1.0)
        print("Bidirectional chat send succeeded")

        if alice.sessions[bob_id.peer_id].role == "initiator":
            await alice.drop_peer(bob_id.peer_id)
        else:
            await bob.drop_peer(alice_id.peer_id)

        await wait_until(
            lambda: bool(alice.sessions[bob_id.peer_id].connected and bob.sessions[alice_id.peer_id].connected),
            timeout=240,
            label="post-drop onion reconnect",
        )
        print("Onion reconnect succeeded")
    finally:
        alice.stop()
        bob.stop()
        await asyncio.gather(alice_task, bob_task, return_exceptions=True)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run onion-mode PP2P smoke test")
    parser.add_argument("--base-dir", default="onion_smoketest_state")
    parser.add_argument("--tor-bin", default=None)
    args = parser.parse_args()

    base = Path(args.base_dir).resolve()
    print(json.dumps({"base_dir": str(base)}, indent=2))
    asyncio.run(run_test(base, tor_bin=args.tor_bin))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
