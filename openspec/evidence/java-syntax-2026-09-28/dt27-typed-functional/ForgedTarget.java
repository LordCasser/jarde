package dt27;

import java.util.function.Function;

public class ForgedTarget {
    public static Function<String, String> wrong() { return String::trim; }

    public static void main(String[] args) {
        System.out.println("value=" + wrong().apply(" x "));
    }
}
