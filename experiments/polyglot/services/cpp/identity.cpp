#include "identity.hpp"
#include "azimuth.hpp"

AZIMUTH_REALIZES("polyglot/identity", "service-identifies-implementation-language")
const char* identity() {
    return "cpp";
}
