package dev.drim.azimuth.emit;

import com.sun.source.tree.CompilationUnitTree;
import com.sun.source.tree.MethodTree;
import com.sun.source.util.JavacTask;
import com.sun.source.util.TreePathScanner;
import com.sun.source.util.Trees;
import dev.drim.azimuth.Azimuth;
import java.io.IOException;
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
import java.util.Arrays;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import javax.tools.JavaCompiler;
import javax.tools.StandardJavaFileManager;
import javax.tools.ToolProvider;

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
        List<Entry> checks = new ArrayList<>();
        List<Entry> implementations = new ArrayList<>();
        List<Entry> artifacts = new ArrayList<>();
        URL[] urls = options.classRoots().stream().map(Main::url).toArray(URL[]::new);
        try (URLClassLoader loader = new URLClassLoader(urls, Main.class.getClassLoader())) {
            for (String className : classNames(options.classRoots())) {
                Class<?> type = Class.forName(className, false, loader);
                if (type.isSynthetic() || type.isAnonymousClass() || type.isLocalClass()) continue;
                Path source = sourceFor(type, sources);
                String file = options.root().toAbsolutePath().normalize()
                        .relativize(source.toAbsolutePath().normalize()).toString().replace('\\', '/');
                String lang = source.toString().endsWith(".kt") ? "kotlin" : "java";
                String fileFingerprint = fingerprint(source);
                collect(type, type.getName(), type.getName(), file, lang,
                        fileFingerprint, fileFingerprint,
                        realizes, checks, implementations, artifacts);
                for (Method method : type.getDeclaredMethods()) {
                    if (method.isSynthetic() || method.isBridge()) continue;
                    String site = type.getName() + "." + method.getName();
                    String mechanismSite = methodSite(method);
                    boolean implementsMechanism = method
                            .getAnnotationsByType(Azimuth.ImplementsMechanism.class).length > 0;
                    String siteFingerprint = method
                            .getAnnotationsByType(Azimuth.ImplementsCheck.class).length > 0
                            ? siteFingerprint(source, method)
                            : implementsMechanism ? fileFingerprint : null;
                    collect(method, site, mechanismSite, file, lang, fileFingerprint,
                            siteFingerprint, realizes, checks, implementations, artifacts);
                }
            }
        }
        realizes.sort(Entry.ORDER);
        checks.sort(Entry.ORDER);
        implementations.sort(Entry.ORDER);
        artifacts.sort(Entry.ORDER);
        return manifest(realizes, checks, implementations, artifacts);
    }

    private static void collect(
            AnnotatedElement element,
            String site,
            String mechanismSite,
            String file,
            String lang,
            String fileFingerprint,
            String siteFingerprint,
            List<Entry> realizes,
            List<Entry> checks,
            List<Entry> implementations,
            List<Entry> artifacts) {
        for (Azimuth.Realizes annotation : element.getAnnotationsByType(Azimuth.Realizes.class)) {
            realizes.add(Entry.relation(
                    annotation.spec(), annotation.scenario(), site, file, lang, fileFingerprint));
        }
        for (Azimuth.ImplementsCheck annotation
                : element.getAnnotationsByType(Azimuth.ImplementsCheck.class)) {
            if (siteFingerprint == null) {
                throw new IllegalArgumentException(
                        site + ": ImplementsCheck requires an exact source fingerprint");
            }
            checks.add(Entry.checkImplementation(
                    annotation.value(), site, file, lang, siteFingerprint));
        }
        Azimuth.ImplementsMechanism[] mechanismAnnotations =
                element.getAnnotationsByType(Azimuth.ImplementsMechanism.class);
        validateMechanismTarget(mechanismSite, mechanismAnnotations.length);
        for (Azimuth.ImplementsMechanism annotation : mechanismAnnotations) {
            if (siteFingerprint == null) {
                throw new IllegalArgumentException(
                        mechanismSite
                                + ": ImplementsMechanism requires an exact source fingerprint");
            }
            String kind = lang + "-symbol";
            String binding = kind + ":" + mechanismSite;
            implementations.add(Entry.implementation(annotation.spec(), annotation.mechanism(),
                    mechanismSite, binding, file, lang, siteFingerprint));
            artifacts.add(Entry.artifact(binding, kind, file));
        }
    }

    static void validateMechanismTarget(String site, int targetCount) {
        if (targetCount > 1) {
            throw new IllegalArgumentException(
                    site + ": one qualified site cannot implement several mechanisms");
        }
    }

    private static String methodSite(Method method) {
        return method.getDeclaringClass().getName() + "." + method.getName()
                + methodDescriptor(method);
    }

    private static String methodDescriptor(Method method) {
        return "(" + String.join("", Arrays.stream(method.getParameterTypes())
                .map(Main::typeDescriptor).toList()) + ")" + typeDescriptor(method.getReturnType());
    }

    private static String typeDescriptor(Class<?> type) {
        if (type.isArray()) return type.getName().replace('.', '/');
        if (!type.isPrimitive()) return "L" + type.getName().replace('.', '/') + ";";
        if (type == void.class) return "V";
        if (type == boolean.class) return "Z";
        if (type == byte.class) return "B";
        if (type == char.class) return "C";
        if (type == short.class) return "S";
        if (type == int.class) return "I";
        if (type == long.class) return "J";
        if (type == float.class) return "F";
        if (type == double.class) return "D";
        throw new IllegalArgumentException("unsupported JVM type " + type.getName());
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
        return sha256Fingerprint(Files.readString(path, StandardCharsets.UTF_8));
    }

    private static String hash(String source) {
        try {
            return java.util.HexFormat.of().formatHex(
                    MessageDigest.getInstance("SHA-256")
                            .digest(source.getBytes(StandardCharsets.UTF_8)));
        } catch (NoSuchAlgorithmException error) {
            throw new IllegalStateException(error);
        }
    }

    private static String siteFingerprint(Path source, Method method) throws IOException {
        String content = Files.readString(source, StandardCharsets.UTF_8);
        String siteSource = source.toString().endsWith(".java")
                ? javaMethodSource(source, content, method)
                : kotlinMethodSource(content, method);
        return sha256Fingerprint(siteSource);
    }

    private static String sha256Fingerprint(String source) {
        return "sha256:" + hash(source);
    }

    private static String javaMethodSource(Path source, String content, Method method)
            throws IOException {
        JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
        if (compiler == null) {
            throw new IllegalArgumentException("a JDK is required to fingerprint " + source);
        }
        List<String> matches = new ArrayList<>();
        try (StandardJavaFileManager files = compiler.getStandardFileManager(null, null,
                StandardCharsets.UTF_8)) {
            JavacTask task = (JavacTask) compiler.getTask(
                    null, files, null, List.of("-proc:none"), null,
                    files.getJavaFileObjects(source));
            for (CompilationUnitTree unit : task.parse()) {
                Trees trees = Trees.instance(task);
                new TreePathScanner<Void, Void>() {
                    @Override
                    public Void visitMethod(MethodTree tree, Void unused) {
                        if (tree.getName().contentEquals(method.getName())
                                && tree.getParameters().size() == method.getParameterCount()) {
                            long start = trees.getSourcePositions().getStartPosition(unit, tree);
                            long end = trees.getSourcePositions().getEndPosition(unit, tree);
                            if (start >= 0 && end > start && end <= content.length()) {
                                matches.add(content.substring((int) start, (int) end));
                            }
                        }
                        return super.visitMethod(tree, unused);
                    }
                }.scan(unit, null);
            }
        }
        if (matches.size() != 1) {
            throw new IllegalArgumentException(
                    "no unique enclosing source site for "
                            + method.getDeclaringClass().getName() + "." + method.getName());
        }
        return matches.get(0);
    }

    private static String kotlinMethodSource(String content, Method method) {
        String marker = "fun " + method.getName();
        int name = content.indexOf(marker);
        if (name < 0 || content.indexOf(marker, name + marker.length()) >= 0) {
            throw new IllegalArgumentException(
                    "no unique enclosing Kotlin source site for "
                            + method.getDeclaringClass().getName() + "." + method.getName());
        }
        int start = content.lastIndexOf('\n', name);
        start = start < 0 ? 0 : start + 1;
        while (start > 0) {
            int previousEnd = start - 1;
            int previousStart = content.lastIndexOf('\n', previousEnd - 1) + 1;
            String previous = content.substring(previousStart, previousEnd).stripLeading();
            if (!previous.startsWith("@")) break;
            start = previousStart;
        }
        int body = content.indexOf('{', name + marker.length());
        int expression = content.indexOf('=', name + marker.length());
        if (expression >= 0 && (body < 0 || expression < body)) {
            int end = content.indexOf('\n', expression);
            return content.substring(start, end < 0 ? content.length() : end);
        }
        if (body < 0) {
            throw new IllegalArgumentException("no method body for " + method.getName());
        }
        int depth = 0;
        for (int index = body; index < content.length(); index++) {
            char character = content.charAt(index);
            if (character == '{') depth++;
            if (character == '}' && --depth == 0) {
                return content.substring(start, index + 1);
            }
        }
        throw new IllegalArgumentException("unterminated method body for " + method.getName());
    }

    private static String manifest(
            List<Entry> realizes,
            List<Entry> checks,
            List<Entry> implementations,
            List<Entry> artifacts) {
        return "{\n"
                + "  \"realizes\": " + array(realizes) + ",\n"
                + "  \"check_implementations\": " + array(checks) + ",\n"
                + "  \"mechanism_implementations\": " + array(implementations) + ",\n"
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

        static Entry checkImplementation(
                String check, String site, String file, String lang, String fingerprint) {
            return entry("check", check, "site", site, "file", file, "lang", lang,
                    "source_fingerprint", fingerprint);
        }

        static Entry implementation(String spec, String mechanism, String site, String binding,
                String file, String lang, String fingerprint) {
            return entry("spec", spec, "mechanism", mechanism, "site", site, "binding", binding,
                    "file", file, "lang", lang, "source_fingerprint", fingerprint);
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
