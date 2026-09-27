package cf05;

public class Runner {
    public static void main(String[] args) {
        ConversionCases probe = new ConversionCases();
        for (boolean flag : new boolean[] {true, false}) {
            System.out.println(ConversionCases.asInt(flag));
            System.out.println(ConversionCases.asLong(flag));
            System.out.println(ConversionCases.asByte(flag));
            System.out.println(ConversionCases.asFloat(flag));
            System.out.println(ConversionCases.asDouble(flag));
            System.out.println(probe.castByte(flag));
            System.out.println(probe.byteField(flag));
            System.out.println(probe.castShort(flag));
            System.out.println(probe.shortField(flag));
            System.out.println(probe.shortConstant(flag));
        }
    }
}
