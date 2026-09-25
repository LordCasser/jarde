public class FinalFailureRunner {
 public static void main(String[]args){FinalFailureSupport.mode=Integer.parseInt(args[0]);for(int i=0;i<2;i++)try{System.out.println(i+":value:"+FinalFailure.value()+":"+FinalFailureSupport.calls+":"+FinalFailureSupport.trace);}catch(Throwable e){System.out.println(i+":"+e.getClass().getName()+":"+(e.getCause()==FinalFailureSupport.FAILURE)+":"+FinalFailureSupport.calls+":"+FinalFailureSupport.trace);}}
}
