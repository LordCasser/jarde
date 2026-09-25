public class DeferredValueOrderRunner {
 interface Task{Object run();}
 static final OrderValue HOLDER=new OrderValue();
 static void reset(int mode){
  DeferredEffects.trace=0;DeferredEffects.mode=mode;DeferredEffects.field=5;
  DeferredEffects.divisor=2;DeferredEffects.longDivisor=2;DeferredEffects.shared=new int[]{7};
  HOLDER.value=5;DeferredEffects.currentHolder=HOLDER;
 }
 static void run(String name,int mode,Task task){
  reset(mode);
  try{System.out.println(name+mode+":"+task.run()+":"+DeferredEffects.trace);}
  catch(RuntimeException e){System.out.println(name+mode+":"+e.getClass().getName()+":"+(e==DeferredEffects.FAILURE)+":"+DeferredEffects.trace);}
 }
 static void runZero(String name,int mode,Task task){
  reset(mode);DeferredEffects.divisor=0;DeferredEffects.longDivisor=0;
  try{System.out.println(name+mode+":"+task.run()+":"+DeferredEffects.trace);}
  catch(RuntimeException e){System.out.println(name+mode+":"+e.getClass().getName()+":"+(e==DeferredEffects.FAILURE)+":"+DeferredEffects.trace);}
 }
 public static void main(String[] args){
  for(int mode=0;mode<3;mode++){
   run("call",mode,()->DeferredValueOrder.call());
   run("field",mode,()->DeferredValueOrder.field());
   run("array",mode,()->DeferredValueOrder.array(DeferredEffects.shared));
   run("instance",mode,()->DeferredValueOrder.instanceField(DeferredEffects.currentHolder));
   run("arrayLength",mode,()->DeferredValueOrder.arrayLength(DeferredEffects.shared));
   run("newArray",mode,()->DeferredValueOrder.newArray(2).length);
   run("anewArray",mode,()->DeferredValueOrder.anewArray(2).length);
   run("multiArray",mode,()->DeferredValueOrder.multiArray(2).length);
   run("goodCast",mode,()->DeferredValueOrder.cast("yes"));
   run("badCast",mode,()->DeferredValueOrder.cast(new Object()));
   run("divInt",mode,()->DeferredValueOrder.divInt(10));
   run("remInt",mode,()->DeferredValueOrder.remInt(10));
   run("divLong",mode,()->DeferredValueOrder.divLong(10L));
   run("remLong",mode,()->DeferredValueOrder.remLong(10L));
   run("direct",mode,()->DeferredValueOrder.direct()!=null);
  }
  for(int mode=0;mode<3;mode++){
   run("newArrayZero",mode,()->DeferredValueOrder.newArray(-1).length);
   run("anewArrayZero",mode,()->DeferredValueOrder.anewArray(-1).length);
   run("multiArrayZero",mode,()->DeferredValueOrder.multiArray(-1).length);
   runZero("divZero",mode,()->DeferredValueOrder.divInt(1));
   runZero("remZero",mode,()->DeferredValueOrder.remInt(1));
   runZero("divLongZero",mode,()->DeferredValueOrder.divLong(1L));
   runZero("remLongZero",mode,()->DeferredValueOrder.remLong(1L));
   run("nullArrayLength",mode,()->DeferredValueOrder.arrayLength(null));
   run("instanceNull",mode,()->DeferredValueOrder.instanceField(null));
  }
  run("nested",0,()->DeferredValueOrder.nested());
  run("branchTrue",0,()->DeferredValueOrder.branch(DeferredEffects.shared,true));
  run("branchFalse",0,()->DeferredValueOrder.branch(DeferredEffects.shared,false));
  run("prefixNull",0,()->DeferredValueOrder.prefix(null));
  run("prefixValue",0,()->DeferredValueOrder.prefix(DeferredEffects.shared));
 }
}
