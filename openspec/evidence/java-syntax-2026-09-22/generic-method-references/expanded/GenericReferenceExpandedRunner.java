import java.util.function.Function;
import java.util.function.IntUnaryOperator;
import java.util.function.Supplier;
import java.util.function.ToIntBiFunction;
import java.util.function.ToIntFunction;

public class GenericReferenceExpandedRunner {
 static void print(String name,Object value){System.out.println(name+":"+value);}
 static void fail(String name,Throwable error){System.out.println(name+":"+error.getClass().getName());}
 static void run(String name,java.util.concurrent.Callable<Object> call){try{print(name,call.call());}catch(Throwable error){fail(name,error);}}
 public static void main(String[]args){
  ToIntFunction<String> strings=GenericReferenceExpanded.staticStrings();
  run("staticString",()->strings.applyAsInt("text"));
  run("staticNull",()->strings.applyAsInt(null));
  run("staticInteger",()->((ToIntFunction)strings).applyAsInt(Integer.valueOf(7)));
  ToIntFunction<String[]> stringArrays=GenericReferenceExpanded.staticStringArrays();
  run("staticStringArray",()->stringArrays.applyAsInt(new String[]{"text"}));
  run("staticNullArray",()->stringArrays.applyAsInt(null));
  run("staticObjectArray",()->((ToIntFunction)stringArrays).applyAsInt(new Object[]{"text"}));
  ToIntFunction<Object[]> objectArrays=GenericReferenceExpanded.staticObjectArrays();
  run("staticObjectArrayTarget",()->objectArrays.applyAsInt(new Object[]{"text"}));
  IntUnaryOperator ints=GenericReferenceExpanded.staticInts();
  run("staticInt",()->ints.applyAsInt(7));
  Function<Integer,Integer> boxedInts=GenericReferenceExpanded.staticBoxedInts();
  run("staticBoxedInt",()->boxedInts.apply(7));

  ToIntBiFunction<ReferenceHelper,String> unbound=GenericReferenceExpanded.unboundStrings();
  run("unboundString",()->unbound.applyAsInt(new ReferenceHelper(),"text"));
  run("unboundNull",()->unbound.applyAsInt(new ReferenceHelper(),null));
  run("unboundInteger",()->((ToIntBiFunction)unbound).applyAsInt(new ReferenceHelper(),Integer.valueOf(7)));
  ToIntBiFunction<ReferenceHelper,String[]> unboundArrays=GenericReferenceExpanded.unboundArrays();
  run("unboundArray",()->unboundArrays.applyAsInt(new ReferenceHelper(),new String[]{"text"}));
  run("unboundObjectArray",()->((ToIntBiFunction)unboundArrays).applyAsInt(new ReferenceHelper(),new Object[]{"text"}));
  Function<String,Integer> bound=GenericReferenceExpanded.boundStrings();
  run("boundString",()->bound.apply("text"));
  Function<String[],Integer> boundArrays=GenericReferenceExpanded.boundArrays();
  run("boundArray",()->boundArrays.apply(new String[]{"text"}));
  run("boundNullReceiver",()->GenericReferenceExpanded.boundNull());

  Function<String,ReferenceBox> constructor=GenericReferenceExpanded.constructor();
  run("constructor",()->constructor.apply("text").value());
  Supplier<String> text=GenericReferenceExpanded.noArgString();
  run("noArgString",text::get);
  Supplier<CharSequence> chars=GenericReferenceExpanded.noArgCharSequence();
  run("noArgCharSequence",chars::get);
  Supplier<String> generic=GenericReferenceExpanded.noArgGenericString();
  run("noArgGenericString",generic::get);
  Supplier rawGeneric=GenericReferenceExpanded.noArgGenericString();
  run("noArgGenericRaw",rawGeneric::get);
 }
}
