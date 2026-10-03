#pragma once

#define AZIMUTH_REALIZES(claim) \
    [[clang::annotate("azimuth|realizes|" claim)]]

#define AZIMUTH_IMPLEMENTS_CHECK(check) \
    [[clang::annotate("azimuth|implements-check|" check)]]

#define AZIMUTH_IMPLEMENTS_MECHANISM(mechanism) \
    [[clang::annotate("azimuth|implements-mechanism|" mechanism)]]
