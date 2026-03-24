#pragma once

#include <string>

namespace pp2p {

std::string generate_identity_json();
std::string peer_id_from_public_key_b64(const std::string &public_key_b64);

}  // namespace pp2p
