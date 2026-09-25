import java.util.function.ToIntFunction;

public class ArrayRunner {
 static void run(String name,java.util.concurrent.Callable<Object> call){try{System.out.println(name+":"+call.call());}catch(Throwable error){System.out.println(name+":"+error.getClass().getName());}}
 public static void main(String[]args){
  ToIntFunction<String[]> strings=ArrayProbe.strings();
  run("string",()->strings.applyAsInt(new String[]{"text"}));
  run("null",()->strings.applyAsInt(null));
  run("object",()->((ToIntFunction)strings).applyAsInt(new Object[]{"text"}));
  ToIntFunction<Object[]> objects=ArrayProbe.objects();
  run("objectTarget",()->objects.applyAsInt(new Object[]{"text"}));
 }
}
