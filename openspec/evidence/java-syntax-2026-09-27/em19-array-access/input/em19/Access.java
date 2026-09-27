package em19;
public class Access {
  public static int at(int i) {
    int[] values = new int[]{1, 2, 3, 5};
    return values[i];
  }
  public static int dimensions(int i) {
    int[][] values = new int[i][i + 1];
    return values.length;
  }
  public static int[] reverseNegate(int[] arr) {
    int len = arr.length;
    int[] result = new int[len];
    int i = 0;
    int k = len;
    while (k != 0) {
      int value = arr[i];
      k--;
      int tmp = -value;
      i++;
      result[k] = tmp * 5;
    }
    return result;
  }
}
