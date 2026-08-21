package dev.drim.azimuth;

import java.lang.annotation.ElementType;
import java.lang.annotation.Repeatable;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

public final class Azimuth {
    private Azimuth() {}

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
    @Target(ElementType.METHOD)
    @Repeatable(CheckImplementations.class)
    public @interface ImplementsCheck { String value(); }

    @Retention(RetentionPolicy.RUNTIME)
    @Target(ElementType.METHOD)
    public @interface CheckImplementations { ImplementsCheck[] value(); }

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

}
