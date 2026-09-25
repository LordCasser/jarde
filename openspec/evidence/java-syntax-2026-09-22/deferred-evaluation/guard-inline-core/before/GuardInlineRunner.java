public class GuardInlineRunner {
 interface Task{Object run();}
 static void test(String name,int mode,Task t){GuardEffects.trace=0;GuardEffects.held=0;GuardEffects.mode=mode;try{System.out.println(name+":"+mode+":"+t.run()+":"+GuardEffects.trace+":"+GuardEffects.held);}catch(Throwable e){System.out.println(name+":"+mode+":"+e.getClass().getName()+":"+(e==GuardEffects.FAIL)+":"+e.getSuppressed().length+":"+GuardEffects.trace+":"+GuardEffects.held);}}
 public static void main(String[] args){for(int m=0;m<7;m++){test("lock",m,()->{GuardInlineCore.locked();return "done";});test("return",m,()->GuardInlineCore.returned());}}
}
