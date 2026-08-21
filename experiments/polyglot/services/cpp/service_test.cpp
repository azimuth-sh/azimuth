#include "identity.hpp"
#include <cassert>
#include <string>

int main() {
    assert(std::string(identity()) == "cpp");
}
