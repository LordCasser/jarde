public class ShiftDistanceBoundary {
 public static int intLeftLongDistance(int value,long distance){return value << (int) distance;}
 public static int intRightLongDistance(int value,long distance){return value >> (int) distance;}
 public static int intUnsignedLongDistance(int value,long distance){return value >>> (int) distance;}
 public static int intLongDistanceLocal(int value,long distance){int shifted=value << (int) distance;return shifted;}
}
