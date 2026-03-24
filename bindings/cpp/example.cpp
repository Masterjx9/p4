#include <iostream>
#include <string>

#include "include/pp2p_core_cpp.hpp"

int main() {
    std::string identity_json = pp2p::generate_identity_json();
    std::cout << identity_json << "\n";
    return 0;
}
