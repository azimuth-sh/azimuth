#pragma once

#define AZIMUTH_REALIZES(spec, claim) \
    [[clang::annotate("azimuth|realizes|" spec "|" claim)]]

#define AZIMUTH_IMPLEMENTS_CHECK(check) \
    [[clang::annotate("azimuth|implements-check|" check)]]

#define AZIMUTH_IMPLEMENTS_MECHANISM(spec, mechanism) \
    [[clang::annotate("azimuth|implements-mechanism|" spec "|" mechanism)]]
