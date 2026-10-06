import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/// A type-use annotation, retained so the implementing class carries the class-level
/// `RuntimeInvisibleTypeAnnotations` attribute that keeps the generic header unpublished.
@Target(ElementType.TYPE_USE)
@Retention(RetentionPolicy.CLASS)
public @interface Mark {
}
