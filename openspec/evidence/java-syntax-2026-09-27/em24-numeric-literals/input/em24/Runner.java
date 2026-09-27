package em24;

import java.util.Arrays;

public final class Runner {
    private Runner() {}

    public static void main(String[] args) {
        System.out.println(Arrays.toString(Numbers.bytes()));
        System.out.println(Arrays.toString(Numbers.shorts()));
        System.out.println(Arrays.toString(Numbers.ints()));
        System.out.println(Arrays.toString(Numbers.longs()));
        StringBuilder floats = new StringBuilder();
        for (float value : Numbers.floats()) {
            if (floats.length() != 0) floats.append(':');
            floats.append(Integer.toHexString(Float.floatToRawIntBits(value)));
        }
        System.out.println(floats);
        StringBuilder doubles = new StringBuilder();
        for (double value : Numbers.doubles()) {
            if (doubles.length() != 0) doubles.append(':');
            doubles.append(Long.toHexString(Double.doubleToRawLongBits(value)));
        }
        System.out.println(doubles);
    }
}
