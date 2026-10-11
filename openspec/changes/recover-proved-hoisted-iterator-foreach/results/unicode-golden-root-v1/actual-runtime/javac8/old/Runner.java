import java.util.*;
public class Runner {
 static void run(String name, List xs) { try { System.out.println(name+"="+SB.cond(xs)); } catch(Throwable t) { System.out.println(name+"="+t.getClass().getName()); } }
 public static void main(String[] args) { SB.main(args); run("empty",Collections.emptyList()); run("null",null); run("strings",Arrays.asList("a","bb","ccc","d")); run("null-element",Arrays.asList("a",null,"bb")); run("wrong-element",Arrays.asList("a",Integer.valueOf(7),"bb")); }
}
