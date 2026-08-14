package dev.drim.azimuth.emit;

import dev.drim.azimuth.Azimuth;
import java.io.IOException;
import java.lang.annotation.Annotation;
import java.lang.reflect.AnnotatedElement;
import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public final class Main {
    private Main() {}

    public static void main(String[] arguments) {
        try {
            Options options = Options.parse(arguments);
            Files.createDirectories(options.output().toAbsolutePath().getParent());
            Files.writeString(options.output(), emit(options), StandardCharsets.UTF_8);
        } catch (IllegalArgumentException | IOException | ReflectiveOperationException error) {
            System.err.println("azimuth-emit-jvm: " + error.getMessage());
            System.exit(2);
        }
    }

    static String emit(Options options)
            throws IOException, ReflectiveOperationException {
        Map<String, Path> sources = sourceFiles(options.sourceRoots());
        List<Entry> realizes = new ArrayList<>();
        List<Entry> covers = new ArrayList<>();
        List<Entry> implementations = new ArrayList<>();
        List<Entry> mechanismCovers = new ArrayList<>();
        List<Entry> artifacts = new ArrayList<>();
        URL[] urls = options.classRoots().stream().map(Main::url).toArray(URL[]::new);
        try (URLClassLoader loader = new URLClassLoader(urls, Main.class.getClassLoader())) {
            for (String className : classNames(options.classRoots())) {
                Class<?> type = Class.forName(className, false, loader);
                Path source = sourceFor(type, sources);
                String file = options.root().toAbsolutePath().normalize()
                        .relativize(source.toAbsolutePath().normalize()).toString().replace('\\', '/');
                String lang = source.toString().endsWith(".kt") ? "kotlin" : "java";
                String fingerprint = fingerprint(source);
                collect(type, type.getName(), file, lang, fingerprint,
                        realizes, covers, implementations, mechanismCovers, artifacts);
                for (Method method : type.getDeclaredMethods()) {
                    collect(method, type.getName() + "." + method.getName(), file, lang, fingerprint,
                            realizes, covers, implementations, mechanismCovers, artifacts);
                }
            }
        }
        realizes.sort(Entry.ORDER);
        covers.sort(Entry.ORDER);
        implementations.sort(Entry.ORDER);
        mechanismCovers.sort(Entry.ORDER);
        artifacts.sort(Entry.ORDER);
        return manifest(realizes, covers, implementations, mechanismCovers, artifacts);
    }

    private static void collect(
            AnnotatedElement element,
            String site,
            String file,
            String lang,
            String fingerprint,
            List<Entry> realizes,
            List<Entry> covers,
            List<Entry> implementations,
            List<Entry> mechanismCovers,
            List<Entry> artifacts) {
        for (Azimuth.Realizes annotation : element.getAnnotationsByType(Azimuth.Realizes.class)) {
            realizes.add(Entry.relation(annotation.spec(), annotation.scenario(), site, file, lang, fingerprint));
        }
        for (Azimuth.Covers annotation : element.getAnnotationsByType(Azimuth.Covers.class)) {
            covers.add(Entry.cover(annotation.spec(), annotation.scenario(), site, file, lang,
                    fingerprint, annotation.scope().name(), annotation.quantification().name(), oracle(annotation.oracle())));
        }
        for (Azimuth.ImplementsMechanism annotation
                : element.getAnnotationsByType(Azimuth.ImplementsMechanism.class)) {
            String binding = "jvm-symbol:" + site;
            implementations.add(Entry.implementation(annotation.spec(), annotation.mechanism(), binding,
                    file, lang, fingerprint));
            artifacts.add(Entry.artifact(binding, "jvm-symbol", file));
        }
        for (Azimuth.CoversMechanism annotation
                : element.getAnnotationsByType(Azimuth.CoversMechanism.class)) {
            mechanismCovers.add(Entry.mechanismCover(annotation.spec(), annotation.mechanism(), site,
                    file, lang, fingerprint, annotation.scope().name(),
                    annotation.quantification().name(), oracle(annotation.oracle())));
        }
    }

    private static String oracle(Azimuth.Oracle value) {
        return value == Azimuth.Oracle.model_based ? "model-based" : value.name();
    }

    private static Map<String, Path> sourceFiles(List<Path> roots) throws IOException {
        Map<String, Path> sources = new HashMap<>();
        for (Path root : roots) {
            try (var paths = Files.walk(root)) {
                for (Path path : paths.filter(Files::isRegularFile)
                        .filter(candidate -> candidate.toString().endsWith(".java")
                                || candidate.toString().endsWith(".kt")).toList()) {
                    String sourceAddress = root.relativize(path).toString().replace('\\', '/');
                    Path previous = sources.putIfAbsent(sourceAddress, path);
                    if (previous != null) {
                        throw new IllegalArgumentException(
                                "source address is ambiguous: " + previous + " and " + path);
                    }
                }
            }
        }
        return sources;
    }

    private static Path sourceFor(Class<?> type, Map<String, Path> sources) {
        String address = type.getName().split("\\$")[0].replace('.', '/');
        List<String> candidates = List.of(address + ".java", address + ".kt",
                address.endsWith("Kt") ? address.substring(0, address.length() - 2) + ".kt" : "");
        return candidates.stream().filter(sources::containsKey).findFirst().map(sources::get)
                .orElseThrow(() -> new IllegalArgumentException("no unique source for " + type.getName()));
    }

    private static List<String> classNames(List<Path> roots) throws IOException {
        List<String> names = new ArrayList<>();
        for (Path root : roots) {
            try (var paths = Files.walk(root)) {
                paths.filter(path -> path.toString().endsWith(".class"))
                        .filter(path -> !path.getFileName().toString().equals("module-info.class"))
                        .filter(path -> !path.getFileName().toString().contains("$"))
                        .forEach(path -> names.add(root.relativize(path).toString()
                                .replace('\\', '.').replace('/', '.').replaceAll("\\.class$", "")));
            }
        }
        names.sort(String::compareTo);
        return names;
    }

    private static URL url(Path path) {
        try {
            return path.toUri().toURL();
        } catch (IOException error) {
            throw new IllegalArgumentException(error);
        }
    }

    private static String fingerprint(Path path) throws IOException {
        try {
            return java.util.HexFormat.of().formatHex(
                    MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(path)));
        } catch (NoSuchAlgorithmException error) {
            throw new IllegalStateException(error);
        }
    }

    private static String manifest(
            List<Entry> realizes,
            List<Entry> covers,
            List<Entry> implementations,
            List<Entry> mechanismCovers,
            List<Entry> artifacts) {
        return "{\n"
                + "  \"realizes\": " + array(realizes) + ",\n"
                + "  \"covers\": " + array(covers) + ",\n"
                + "  \"mechanism_implementations\": " + array(implementations) + ",\n"
                + "  \"mechanism_covers\": " + array(mechanismCovers) + ",\n"
                + "  \"class_members\": [],\n"
                + "  \"enumerations\": [],\n"
                + "  \"artifacts\": " + array(artifacts) + "\n"
                + "}\n";
    }

    private static String array(List<Entry> entries) {
        if (entries.isEmpty()) return "[]";
        return "[\n    " + String.join(",\n    ", entries.stream().map(Entry::json).toList()) + "\n  ]";
    }

    record Options(Path output, Path root, List<Path> sourceRoots, List<Path> classRoots) {
        static Options parse(String[] arguments) {
            Path output = null;
            Path root = Path.of(".");
            List<Path> sources = new ArrayList<>();
            List<Path> classes = new ArrayList<>();
            for (int index = 0; index < arguments.length; index++) {
                switch (arguments[index]) {
                    case "--output", "-o" -> output = Path.of(value(arguments, ++index));
                    case "--root" -> root = Path.of(value(arguments, ++index));
                    case "--source-root" -> sources.add(Path.of(value(arguments, ++index)));
                    case "--classes" -> classes.add(Path.of(value(arguments, ++index)));
                    default -> throw new IllegalArgumentException("unknown option `" + arguments[index] + "`");
                }
            }
            if (output == null || sources.isEmpty() || classes.isEmpty()) {
                throw new IllegalArgumentException(
                        "usage: azimuth-emit-jvm --output <path> --source-root <dir> --classes <dir>");
            }
            return new Options(output, root, sources, classes);
        }

        private static String value(String[] arguments, int index) {
            if (index >= arguments.length) throw new IllegalArgumentException("option needs a value");
            return arguments[index];
        }
    }

    record Entry(Map<String, String> fields) {
        static final Comparator<Entry> ORDER = Comparator.comparing(Entry::json);

        static Entry relation(String spec, String scenario, String site, String file, String lang, String fingerprint) {
            return entry("spec", spec, "scenario", scenario, "site", site, "file", file,
                    "lang", lang, "source_fingerprint", fingerprint);
        }

        static Entry cover(String spec, String scenario, String site, String file, String lang,
                String fingerprint, String scope, String quantification, String oracle) {
            return entry("spec", spec, "scenario", scenario, "site", site, "file", file,
                    "lang", lang, "source_fingerprint", fingerprint, "scope", scope,
                    "quantification", quantification, "oracle", oracle);
        }

        static Entry implementation(String spec, String mechanism, String binding, String file,
                String lang, String fingerprint) {
            return entry("spec", spec, "mechanism", mechanism, "binding", binding, "file", file,
                    "lang", lang, "source_fingerprint", fingerprint);
        }

        static Entry mechanismCover(String spec, String mechanism, String site, String file,
                String lang, String fingerprint, String scope, String quantification, String oracle) {
            return entry("spec", spec, "mechanism", mechanism, "site", site, "file", file,
                    "lang", lang, "source_fingerprint", fingerprint, "scope", scope,
                    "quantification", quantification, "oracle", oracle);
        }

        static Entry artifact(String id, String kind, String file) {
            return entry("id", id, "kind", kind, "file", file);
        }

        static Entry entry(String... values) {
            Map<String, String> fields = new java.util.LinkedHashMap<>();
            for (int index = 0; index < values.length; index += 2) fields.put(values[index], values[index + 1]);
            return new Entry(fields);
        }

        String json() {
            return "{" + String.join(",", fields.entrySet().stream()
                    .map(item -> "\"" + escape(item.getKey()) + "\":\"" + escape(item.getValue()) + "\"")
                    .toList()) + "}";
        }

        private static String escape(String value) {
            return value.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n");
        }
    }
}
