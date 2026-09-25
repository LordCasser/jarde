import java.util.function.ToIntFunction;
public class GenericReferenceRunner {
 static void run(String n,ToIntFunction fn,Object value){try{System.out.println(n+":"+fn.applyAsInt(value));}catch(RuntimeException e){System.out.println(n+":"+e.getClass().getName());}}
 public static void main(String[]args){ToIntFunction strings=GenericReference.strings();ToIntFunction objects=GenericReference.objects();run("stringText",strings,"text");run("stringNull",strings,null);run("stringInteger",strings,Integer.valueOf(7));run("objectText",objects,"text");run("objectNull",objects,null);run("objectInteger",objects,Integer.valueOf(7));}
}
