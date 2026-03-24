#include "pp2p_core_cpp.hpp"

#include "../../../include/pp2p_core.h"

#include <stdexcept>

namespace pp2p {
namespace {

std::string take_string(char *ptr) {
  if (ptr == nullptr) {
    char *err = pp2p_last_error_message();
    std::string message = err ? std::string(err) : std::string("unknown error");
    pp2p_free_string(err);
    throw std::runtime_error(message);
  }
  std::string value(ptr);
  pp2p_free_string(ptr);
  return value;
}

}  // namespace

std::string generate_identity_json() { return take_string(pp2p_generate_identity_json()); }

std::string peer_id_from_public_key_b64(const std::string &public_key_b64) {
  return take_string(pp2p_peer_id_from_public_key_b64(public_key_b64.c_str()));
}

}  // namespace pp2p
