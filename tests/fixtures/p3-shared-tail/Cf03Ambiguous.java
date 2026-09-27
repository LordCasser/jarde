public class Cf03Ambiguous {
 public static int f(boolean gate,int x,int y) {
  if (gate) {
   if (x == 0) return 100;
   if (y != 0) return 200;
  } else {
   if (x == 0) return 100;
   if (y != 0) return 200;
  }
  return 0;
 }
}
