public class InlineOrderControls {
 public static void voidArgument(){InlineOrderEffects.accept(InlineOrderEffects.value());}
 public static int callArgument(){return InlineOrderEffects.take(InlineOrderEffects.value());}
 public static void discarded(){InlineOrderEffects.value();}
 public static String concatenate(){return "x="+InlineOrderEffects.value();}
 public static InlineHolder construct(){return new InlineHolder(InlineOrderEffects.value());}
 public static int receiverAndArgument(){return InlineOrderEffects.make().read(InlineOrderEffects.value());}
 public static int fieldAndCall(){return InlineOrderEffects.make().value+InlineOrderEffects.value();}
 public static void fieldWrite(){InlineOrderEffects.make().value=InlineOrderEffects.value();}
 public static int local(){int x=InlineOrderEffects.value();return x+1;}
 public static int test(){if(InlineOrderEffects.take(InlineOrderEffects.value())==6)return 1;return 2;}
}
