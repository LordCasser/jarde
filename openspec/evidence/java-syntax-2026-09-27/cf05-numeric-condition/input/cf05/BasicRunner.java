package cf05;

public class BasicRunner {
    public static void main(String[] args) {
        for (boolean flag : new boolean[] {true, false}) {
            System.out.println(ConversionBasic.asInt(flag));
            System.out.println(ConversionBasic.asLong(flag));
            System.out.println(ConversionBasic.asByte(flag));
            System.out.println(ConversionBasic.asFloat(flag));
            System.out.println(ConversionBasic.asDouble(flag));
        }
    }
}
