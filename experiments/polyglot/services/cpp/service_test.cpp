#include "identity.hpp"
#include "azimuth.hpp"

#include <cassert>
#include <string>

AZIMUTH_COVERS("polyglot/identity", "cpp-identifies", "unit", "example", "direct")
int main() {
    assert(std::string(identity()) == "cpp");
}
