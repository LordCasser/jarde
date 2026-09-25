import java.util.function.ToIntFunction;
public class WiderTargetRunner {
 static void run(String name,Object value){WiderTarget.calls=0;ToIntFunction f=WiderTarget.strings();try{System.out.println(name+":"+f.applyAsInt(value)+":"+WiderTarget.calls);}catch(RuntimeException e){System.out.println(name+":"+e.getClass().getName()+":"+WiderTarget.calls);}}
 public static void main(String[]args){run("text","x");run("null",null);run("integer",Integer.valueOf(7));}
}
