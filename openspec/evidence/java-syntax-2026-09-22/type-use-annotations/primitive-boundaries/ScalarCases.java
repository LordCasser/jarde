import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.FIELD, ElementType.METHOD, ElementType.PARAMETER, ElementType.TYPE_USE})
@interface A {}

public class ScalarCases {
    @A int field;

    @A int answer() {
        return 42;
    }

    int echo(@A int value) {
        return value;
    }
}
