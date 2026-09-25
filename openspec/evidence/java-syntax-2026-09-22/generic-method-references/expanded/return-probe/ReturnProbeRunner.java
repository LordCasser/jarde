import java.util.function.Supplier;

public class ReturnProbeRunner {
 static void print(String name,Object value){System.out.println(name+":"+value.getClass().getName()+":"+value);}
 static void run(String name,java.util.concurrent.Callable<Object> call){try{print(name,call.call());}catch(Throwable error){System.out.println(name+":"+error.getClass().getName());}}
 public static void main(String[]args){
  ReturnHelper.current=args.length>0&&args[0].equals("integer")?Integer.valueOf(7):"text";
  Supplier raw=ReturnProbe.genericString();
  run("raw",raw::get);
  Supplier<String> typed=ReturnProbe.genericString();
  run("typed",()->{String value=typed.get();return value;});
 }
}
