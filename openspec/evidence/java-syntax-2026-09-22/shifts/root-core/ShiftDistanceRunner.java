public class ShiftDistanceRunner {
 private static void out(String name,Object value){System.out.println(name+"="+value);}
 public static void main(String[] args){
  for(int value:new int[]{Integer.MIN_VALUE,-1,0,1,Integer.MAX_VALUE})for(long distance:new long[]{-65L,-1L,0L,1L,31L,32L,63L,64L,65L,Long.MAX_VALUE}){
   out("bL:"+value+":"+distance,ShiftDistanceBoundary.intLeftLongDistance(value,distance));
   out("bR:"+value+":"+distance,ShiftDistanceBoundary.intRightLongDistance(value,distance));
   out("bU:"+value+":"+distance,ShiftDistanceBoundary.intUnsignedLongDistance(value,distance));
   out("bLocal:"+value+":"+distance,ShiftDistanceBoundary.intLongDistanceLocal(value,distance));
  }
 }
}
