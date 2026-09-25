public class NumericRefusalEffects {
 public static int calls;public static boolean failing;
 public static float value(float x){calls++;if(failing)throw new IllegalStateException("operand");return x;}
}
