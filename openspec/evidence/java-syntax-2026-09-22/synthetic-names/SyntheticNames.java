import java.util.function.IntUnaryOperator;
public class SyntheticNames {
 public static int repeated(int seed,int value){
  int p0_=seed;
  IntUnaryOperator a=v->v+p0_;
  IntUnaryOperator b=v->v-p0_;
  return a.applyAsInt(value)+b.applyAsInt(value);
 }
 public static void main(String[]args){System.out.println(repeated(7,11));}
}
