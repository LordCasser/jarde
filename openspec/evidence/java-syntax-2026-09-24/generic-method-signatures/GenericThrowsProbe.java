import java.io.IOException;

public class GenericThrowsProbe {
    public static <T extends Number> T choose(T value) throws IOException {
        return value;
    }
}
