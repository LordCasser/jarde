import java.io.IOException;
import java.util.List;

public class ComplexMethodProbe {
    public static <T extends Number> List<? extends T>[] choose(
            List<? extends T>[] values) throws IOException {
        if (values.length == 0) {
            throw new IOException();
        }
        return values;
    }
}
