public class S3FloatBitsDev {
    static String trace = "";
    static float observe(float value) { trace += "f"; return value; }
    static String bits(float value) { return Integer.toHexString(Float.floatToRawIntBits(value)); }
    public static String run(int n) {
        trace = "";
        float nanValue = Float.intBitsToFloat(0x7fc12345);
        float nanOld = nanValue;
        nanValue = Float.NaN;
        float nanUpdated = nanValue;
        float positiveZero = +0.0f;
        float positiveOld = positiveZero++;
        float positiveUpdated = ++positiveZero;
        float negativeZero = -0.0f;
        float negativeOld = negativeZero++;
        float negativeUpdated = ++negativeZero;
        String oldFirst = bits(observe(nanOld));
        String oldSecond = bits(nanOld);
        String updatedFirst = bits(observe(nanUpdated));
        String updatedSecond = bits(nanUpdated);
        String posOld = bits(positiveOld), posUpdated = bits(positiveUpdated);
        String negOld = bits(negativeOld), negUpdated = bits(negativeUpdated);
        trace += ":" + n;
        return oldFirst + ":" + oldSecond + ":" + updatedFirst + ":" + updatedSecond + ":" +
                posOld + ":" + posUpdated + ":" + negOld + ":" + negUpdated + ":" + trace;
    }
}
