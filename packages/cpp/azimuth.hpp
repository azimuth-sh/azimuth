#pragma once

#define AZIMUTH_REALIZES(spec, scenario) \
    [[clang::annotate("azimuth|realizes|" spec "|" scenario)]]

#define AZIMUTH_COVERS(spec, scenario, scope, quantification, oracle) \
    [[clang::annotate("azimuth|covers|" spec "|" scenario "|" scope "|" quantification "|" oracle)]]

#define AZIMUTH_IMPLEMENTS_MECHANISM(spec, mechanism) \
    [[clang::annotate("azimuth|implements-mechanism|" spec "|" mechanism)]]

#define AZIMUTH_COVERS_MECHANISM(spec, mechanism, scope, quantification, oracle) \
    [[clang::annotate("azimuth|covers-mechanism|" spec "|" mechanism "|" scope "|" quantification "|" oracle)]]
