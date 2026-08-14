package dev.drim.azimuth;

import java.lang.annotation.ElementType;
import java.lang.annotation.Repeatable;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

public final class Azimuth {
    private Azimuth() {}

    public enum Scope { unit, component, e2e }

    public enum Quantification { example, universal }

    public enum Oracle { direct, golden, relational, metamorphic, model_based, contract }

    @Retention(RetentionPolicy.RUNTIME)
    @Target({ElementType.TYPE, ElementType.METHOD})
    @Repeatable(Realizations.class)
    public @interface Realizes {
        String spec();
        String scenario();
    }

    @Retention(RetentionPolicy.RUNTIME)
    @Target({ElementType.TYPE, ElementType.METHOD})
    public @interface Realizations { Realizes[] value(); }

    @Retention(RetentionPolicy.RUNTIME)
    @Target({ElementType.TYPE, ElementType.METHOD})
    @Repeatable(Coverage.class)
    public @interface Covers {
        String spec();
        String scenario();
        Scope scope();
        Quantification quantification();
        Oracle oracle() default Oracle.direct;
    }

    @Retention(RetentionPolicy.RUNTIME)
    @Target({ElementType.TYPE, ElementType.METHOD})
    public @interface Coverage { Covers[] value(); }

    @Retention(RetentionPolicy.RUNTIME)
    @Target({ElementType.TYPE, ElementType.METHOD})
    @Repeatable(MechanismImplementations.class)
    public @interface ImplementsMechanism {
        String spec();
        String mechanism();
    }

    @Retention(RetentionPolicy.RUNTIME)
    @Target({ElementType.TYPE, ElementType.METHOD})
    public @interface MechanismImplementations { ImplementsMechanism[] value(); }

    @Retention(RetentionPolicy.RUNTIME)
    @Target({ElementType.TYPE, ElementType.METHOD})
    @Repeatable(MechanismCoverage.class)
    public @interface CoversMechanism {
        String spec();
        String mechanism();
        Scope scope();
        Quantification quantification();
        Oracle oracle() default Oracle.direct;
    }

    @Retention(RetentionPolicy.RUNTIME)
    @Target({ElementType.TYPE, ElementType.METHOD})
    public @interface MechanismCoverage { CoversMechanism[] value(); }
}
