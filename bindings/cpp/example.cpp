#include <iostream>
#include <string>

#include "include/p4_core_cpp.hpp"

int main() {
    std::string identity_json = p4::generate_identity_json();
    std::cout << identity_json << "\n";
    return 0;
}

