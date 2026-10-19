#include "identity.hpp"
#include "azimuth.hpp"

AZIMUTH_REALIZES("service-identifies-implementation-language")
const char* identity() {
    return "cpp";
}
