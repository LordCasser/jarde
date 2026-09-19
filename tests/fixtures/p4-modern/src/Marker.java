/**
 * An annotation whose only target is a record component, so the `Record` attribute itself carries
 * the component's `attribute_info` list (`@Deprecated` does not: its targets are the field and the
 * accessor method, which `javac` writes outside the `Record` attribute).
 */
@java.lang.annotation.Target(java.lang.annotation.ElementType.RECORD_COMPONENT)
@java.lang.annotation.Retention(java.lang.annotation.RetentionPolicy.RUNTIME)
public @interface Marker {}
