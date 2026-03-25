#!/usr/bin/env python3
"""
Local smoke test for P4 protocol without onion relay.

It validates:
1) Two nodes negotiate and connect.
2) Messages can be sent both directions.
3) Forced disconnect triggers automatic reconnect.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import shutil
import time
from pathlib import Path

from p4 import Contact, Rendezvous, RuntimeConfig, P4Node, save_contacts, state_identity


async def wait_until(pred, timeout: float, label: str) -> None:
    deadline = time.monotonic() + timeout
    while not pred():
        if time.monotonic() > deadline:
            raise TimeoutError(f"Timed out waiting for: {label}")
        await asyncio.sleep(0.2)


async def run_test(base_dir: Path) -> None:
    if base_dir.exists():
        shutil.rmtree(base_dir)
    (base_dir / "alice").mkdir(parents=True, exist_ok=True)
    (base_dir / "bob").mkdir(parents=True, exist_ok=True)

    alice_state = base_dir / "alice"
    bob_state = base_dir / "bob"

    alice_id = state_identity(alice_state)
    bob_id = state_identity(bob_state)

    alice_contact = Contact(
        peer_id=bob_id.peer_id,
        public_key_b64=bob_id.public_key_b64,
        rendezvous=Rendezvous(transport="direct", address="127.0.0.1", port=18102),
        name="bob",
    )
    bob_contact = Contact(
        peer_id=alice_id.peer_id,
        public_key_b64=alice_id.public_key_b64,
        rendezvous=Rendezvous(transport="direct", address="127.0.0.1", port=18101),
        name="alice",
    )
    save_contacts(alice_state, {bob_contact.peer_id: alice_contact})
    save_contacts(bob_state, {alice_contact.peer_id: bob_contact})

    alice_cfg = RuntimeConfig(
        state_dir=alice_state,
        mode="direct",
        signal_host="127.0.0.1",
        signal_port=18101,
        advertise_host="127.0.0.1",
        retry_seconds=2.0,
        stun_server="stun:stun.l.google.com:19302",
        turn_server=None,
        turn_username=None,
        turn_password=None,
        turn_secret=None,
        turn_ttl_seconds=3600,
        turn_user="p4",
        onionrelay_bin=None,
        onionrelay_socks_port=0,
        onionrelay_control_port=0,
        no_stdin=True,
    )
    bob_cfg = RuntimeConfig(
        state_dir=bob_state,
        mode="direct",
        signal_host="127.0.0.1",
        signal_port=18102,
        advertise_host="127.0.0.1",
        retry_seconds=2.0,
        stun_server="stun:stun.l.google.com:19302",
        turn_server=None,
        turn_username=None,
        turn_password=None,
        turn_secret=None,
        turn_ttl_seconds=3600,
        turn_user="p4",
        onionrelay_bin=None,
        onionrelay_socks_port=0,
        onionrelay_control_port=0,
        no_stdin=True,
    )

    alice = P4Node(alice_cfg)
    bob = P4Node(bob_cfg)

    alice_task = asyncio.create_task(alice.run(), name="alice-run")
    bob_task = asyncio.create_task(bob.run(), name="bob-run")

    try:
        await wait_until(
            lambda: alice.sessions.get(bob_id.peer_id) is not None and bob.sessions.get(alice_id.peer_id) is not None,
            timeout=15,
            label="session maps initialized",
        )
        await wait_until(
            lambda: bool(alice.sessions[bob_id.peer_id].connected and bob.sessions[alice_id.peer_id].connected),
            timeout=60,
            label="initial connection",
        )
        print("Initial connection established")

        await alice.send_chat(bob_id.peer_id, "hello-from-alice")
        await bob.send_chat(alice_id.peer_id, "hello-from-bob")
        await asyncio.sleep(1.0)
        print("Bidirectional chat send succeeded")

        # Force drop from whichever side is initiator.
        if alice.sessions[bob_id.peer_id].role == "initiator":
            await alice.drop_peer(bob_id.peer_id)
        else:
            await bob.drop_peer(alice_id.peer_id)

        await wait_until(
            lambda: bool(alice.sessions[bob_id.peer_id].connected and bob.sessions[alice_id.peer_id].connected),
            timeout=60,
            label="post-drop reconnect",
        )
        print("Reconnect succeeded")
    finally:
        alice.stop()
        bob.stop()
        await asyncio.gather(alice_task, bob_task, return_exceptions=True)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run local P4 direct smoke test")
    parser.add_argument("--base-dir", default="smoketest_state")
    args = parser.parse_args()

    base = Path(args.base_dir).resolve()
    print(json.dumps({"base_dir": str(base)}, indent=2))
    asyncio.run(run_test(base))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())



