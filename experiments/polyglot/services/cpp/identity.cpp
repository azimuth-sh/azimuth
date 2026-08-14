#include "identity.hpp"
#include "azimuth.hpp"

AZIMUTH_REALIZES("polyglot/identity", "cpp-identifies")
const char* identity() {
    return "cpp";
}
