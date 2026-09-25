public class ThrowFlowRunner {
 private static void attempt(String name,int mode,Runnable task){
  ThrowFlowEffects.calls=0;ThrowFlowEffects.mode=mode;
  try{task.run();System.out.println(name+"=returned");}
  catch(RuntimeException error){System.out.println(name+"="+error.getClass().getName()+":"+(error instanceof ThrowFlowException ? error.getMessage() : "-")+":"+(error==ThrowFlowEffects.FAIL)+":"+(error==ThrowFlowEffects.VALUE)+":"+ThrowFlowEffects.calls);}
 }
 public static void main(String[] args){
  attempt("constructed",0,()->ThrowFlow.constructed("message"));
  attempt("argumentFailure",1,()->ThrowFlow.constructed("message"));
  attempt("constructorFailure",2,()->ThrowFlow.constructed("message"));
  attempt("nested",0,()->ThrowFlow.nested());
  attempt("problemFailure",3,()->ThrowFlow.nested());
  attempt("wrapperFailure",4,()->ThrowFlow.nested());
  attempt("arrayValue",0,()->ThrowFlow.array(new RuntimeException[]{ThrowFlowEffects.VALUE},0));
  attempt("arrayElementNull",0,()->ThrowFlow.array(new RuntimeException[]{null},0));
  attempt("arrayNull",0,()->ThrowFlow.array(null,0));
  attempt("arrayBounds",0,()->ThrowFlow.array(new RuntimeException[0],0));
  attempt("fieldValue",0,()->ThrowFlow.field(new ThrowFlowHolder(ThrowFlowEffects.VALUE)));
  attempt("fieldNull",0,()->ThrowFlow.field(new ThrowFlowHolder(null)));
  attempt("receiverNull",0,()->ThrowFlow.field(null));
  attempt("first",0,()->ThrowFlow.branch(true,ThrowFlowEffects.VALUE,ThrowFlowEffects.FAIL));
  attempt("second",0,()->ThrowFlow.branch(false,ThrowFlowEffects.VALUE,ThrowFlowEffects.FAIL));
 }
}
