import java.util.function.Supplier;

public class ReturnProbe {
 public static Supplier<String> genericString(){return ReturnHelper::<String>value;}
}
