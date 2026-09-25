public class ThrowFlow {
 public static void constructed(String text){throw new ThrowFlowException(ThrowFlowEffects.argument(text));}
 public static void nested(){throw ThrowFlowEffects.wrapper(ThrowFlowEffects.problem());}
 public static void array(RuntimeException[] errors,int index){throw errors[index];}
 public static void field(ThrowFlowHolder holder){throw holder.error;}
 public static void branch(boolean first,RuntimeException a,RuntimeException b){if(first)throw a;throw b;}
}
