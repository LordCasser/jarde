import java.util.function.BiFunction;
import java.util.function.Function;
import java.util.function.IntUnaryOperator;
import java.util.function.Supplier;
import java.util.function.ToIntBiFunction;
import java.util.function.ToIntFunction;

public class GenericReferenceExpanded {
 public static int staticPick(Object value){return 10;}
 public static int staticPick(String value){return 11;}
 public static int staticPick(Object[] value){return 12;}
 public static int staticPick(String[] value){return 13;}
 public static int staticPick(int value){return 14;}
 public static int staticPick(Integer value){return 15;}

 public static ToIntFunction<String> staticStrings(){return GenericReferenceExpanded::staticPick;}
 public static ToIntFunction<String[]> staticStringArrays(){return GenericReferenceExpanded::staticPick;}
 public static ToIntFunction<Object[]> staticObjectArrays(){return GenericReferenceExpanded::staticPick;}
 public static IntUnaryOperator staticInts(){return GenericReferenceExpanded::staticPick;}
 public static Function<Integer,Integer> staticBoxedInts(){return GenericReferenceExpanded::staticPick;}

 public static ToIntBiFunction<ReferenceHelper,String> unboundStrings(){return ReferenceHelper::pick;}
 public static ToIntBiFunction<ReferenceHelper,String[]> unboundArrays(){return ReferenceHelper::pick;}
 public static Function<String,Integer> boundStrings(){return ReferenceHelper.make()::pick;}
 public static Function<String[],Integer> boundArrays(){return ReferenceHelper.make()::pick;}
 public static Function<String,Integer> boundNull(){return ReferenceHelper.nullReceiver()::pick;}

 public static Function<String,ReferenceBox> constructor(){return ReferenceBox::new;}
 public static Supplier<String> noArgString(){return ReferenceHelper::text;}
 public static Supplier<CharSequence> noArgCharSequence(){return ReferenceHelper::text;}
 public static Supplier<String> noArgGenericString(){return ReferenceHelper::<String>genericText;}
}
