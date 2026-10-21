#pragma once

#define AZIMUTH_REALIZES(requirement) \
    [[clang::annotate("azimuth|realizes|" requirement)]]

#define AZIMUTH_IMPLEMENTS_CHECK(check) \
    [[clang::annotate("azimuth|implements-check|" check)]]

#define AZIMUTH_IMPLEMENTS_MECHANISM(mechanism) \
    [[clang::annotate("azimuth|implements-mechanism|" mechanism)]]
