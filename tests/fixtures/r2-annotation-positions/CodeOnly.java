import java.lang.annotation.*;
@Retention(RetentionPolicy.RUNTIME) @Target(ElementType.TYPE_USE)
@interface CodeMarker {}
class CodeOnly { Object f(Object o) { return (@CodeMarker String)o; } }
