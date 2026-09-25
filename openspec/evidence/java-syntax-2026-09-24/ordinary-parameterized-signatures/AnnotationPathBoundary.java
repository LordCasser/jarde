import java.lang.annotation.*;
import java.util.List;
public class AnnotationPathBoundary {
    @Retention(RetentionPolicy.RUNTIME)
    @Target(ElementType.TYPE_USE)
    public @interface Mark {}
    public static List<@Mark String> identity(List<@Mark String> value) { return value; }
}
