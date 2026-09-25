public class CheckRunner {
 interface Task{Object run();}
 static void run(String n,int mode,Task t){CheckSupport.trace=0;CheckSupport.mode=mode;CheckSupport.shared=new CheckSupport();CheckSupport.shared.value=5;try{System.out.println(n+mode+":"+t.run()+":"+CheckSupport.trace);}catch(RuntimeException e){System.out.println(n+mode+":"+e.getClass().getName()+":"+(e==CheckSupport.FAILURE)+":"+CheckSupport.trace);}}
 public static void main(String[] args){for(int m=0;m<2;m++){
 run("divI0",m,()->DeferredChecks.divI(10,0));run("divI2",m,()->DeferredChecks.divI(10,2));
 run("remI0",m,()->DeferredChecks.remI(10,0));run("remI2",m,()->DeferredChecks.remI(10,2));
 run("divL0",m,()->DeferredChecks.divL(10L,0L));run("divL2",m,()->DeferredChecks.divL(10L,2L));
 run("remL0",m,()->DeferredChecks.remL(10L,0L));run("remL2",m,()->DeferredChecks.remL(10L,2L));
 run("field",m,()->DeferredChecks.field(CheckSupport.shared));run("nullField",m,()->DeferredChecks.field(null));
 run("length",m,()->DeferredChecks.length(new int[]{4,5}));run("nullLength",m,()->DeferredChecks.length(null));
 }}
}
