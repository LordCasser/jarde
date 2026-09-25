public class StructuredRunner {
 interface Task{Object run();}
 static void run(String n,int m,boolean t,Task fn){StructuredSupport.trace=0;StructuredSupport.mode=m;StructuredSupport.test=t;StructuredSupport.field=5;try{System.out.println(n+":"+m+":"+t+":"+fn.run()+":"+StructuredSupport.trace);}catch(RuntimeException e){System.out.println(n+":"+m+":"+t+":"+e.getClass().getName()+":"+(e==StructuredSupport.FAILURE)+":"+StructuredSupport.trace);}}
 public static void main(String[]args){for(int m=0;m<3;m++)for(boolean t:new boolean[]{false,true}){run("branch",m,t,()->DeferredStructured.branch(t));run("prefix",m,t,()->DeferredStructured.prefix());}}
}
