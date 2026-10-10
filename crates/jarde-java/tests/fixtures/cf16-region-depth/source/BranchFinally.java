public class BranchFinally {
    static final int DEPTH = 33;
    static int trace;

    private static void cleanup() {
        trace++;
    }

    static int run(int x) {
        try {
            if (trace + x > 1) {
                trace += 1;
                if (x > 2) {
                    trace += 2;
                    if (x > 3) {
                        trace += 3;
                        if (x > 4) {
                            trace += 4;
                            if (x > 5) {
                                trace += 5;
                                if (x > 6) {
                                    trace += 6;
                                    if (x > 7) {
                                        trace += 7;
                                        if (x > 8) {
                                            trace += 8;
                                            if (x > 9) {
                                                trace += 9;
                                                if (x > 10) {
                                                    trace += 10;
                                                    if (x > 11) {
                                                        trace += 11;
                                                        if (x > 12) {
                                                            trace += 12;
                                                            if (x > 13) {
                                                                trace += 13;
                                                                if (x > 14) {
                                                                    trace += 14;
                                                                    if (x > 15) {
                                                                        trace += 15;
                                                                        if (x > 16) {
                                                                            trace += 16;
                                                                            if (x > 17) {
                                                                                trace += 17;
                                                                                if (x > 18) {
                                                                                    trace += 18;
                                                                                    if (x > 19) {
                                                                                        trace += 19;
                                                                                        if (x > 20) {
                                                                                            trace += 20;
                                                                                            if (x > 21) {
                                                                                                trace += 21;
                                                                                                if (x > 22) {
                                                                                                    trace += 22;
                                                                                                    if (x > 23) {
                                                                                                        trace += 23;
                                                                                                        if (x > 24) {
                                                                                                            trace += 24;
                                                                                                            if (x > 25) {
                                                                                                                trace += 25;
                                                                                                                if (x > 26) {
                                                                                                                    trace += 26;
                                                                                                                    if (x > 27) {
                                                                                                                        trace += 27;
                                                                                                                        if (x > 28) {
                                                                                                                            trace += 28;
                                                                                                                            if (x > 29) {
                                                                                                                                trace += 29;
                                                                                                                                if (x > 30) {
                                                                                                                                    trace += 30;
                                                                                                                                    if (x > 31) {
                                                                                                                                        trace += 31;
                                                                                                                                        if (x > 32) {
                                                                                                                                            trace += 32;
                                                                                                                                            if (x > 33) {
                                                                                                                                                trace += 33;
                                                                                                                                            } else {
                                                                                                                                                trace -= 33;
                                                                                                                                            }
                                                                                                                                        } else {
                                                                                                                                            trace -= 32;
                                                                                                                                        }
                                                                                                                                    } else {
                                                                                                                                        trace -= 31;
                                                                                                                                    }
                                                                                                                                } else {
                                                                                                                                    trace -= 30;
                                                                                                                                }
                                                                                                                            } else {
                                                                                                                                trace -= 29;
                                                                                                                            }
                                                                                                                        } else {
                                                                                                                            trace -= 28;
                                                                                                                        }
                                                                                                                    } else {
                                                                                                                        trace -= 27;
                                                                                                                    }
                                                                                                                } else {
                                                                                                                    trace -= 26;
                                                                                                                }
                                                                                                            } else {
                                                                                                                trace -= 25;
                                                                                                            }
                                                                                                        } else {
                                                                                                            trace -= 24;
                                                                                                        }
                                                                                                    } else {
                                                                                                        trace -= 23;
                                                                                                    }
                                                                                                } else {
                                                                                                    trace -= 22;
                                                                                                }
                                                                                            } else {
                                                                                                trace -= 21;
                                                                                            }
                                                                                        } else {
                                                                                            trace -= 20;
                                                                                        }
                                                                                    } else {
                                                                                        trace -= 19;
                                                                                    }
                                                                                } else {
                                                                                    trace -= 18;
                                                                                }
                                                                            } else {
                                                                                trace -= 17;
                                                                            }
                                                                        } else {
                                                                            trace -= 16;
                                                                        }
                                                                    } else {
                                                                        trace -= 15;
                                                                    }
                                                                } else {
                                                                    trace -= 14;
                                                                }
                                                            } else {
                                                                trace -= 13;
                                                            }
                                                        } else {
                                                            trace -= 12;
                                                        }
                                                    } else {
                                                        trace -= 11;
                                                    }
                                                } else {
                                                    trace -= 10;
                                                }
                                            } else {
                                                trace -= 9;
                                            }
                                        } else {
                                            trace -= 8;
                                        }
                                    } else {
                                        trace -= 7;
                                    }
                                } else {
                                    trace -= 6;
                                }
                            } else {
                                trace -= 5;
                            }
                        } else {
                            trace -= 4;
                        }
                    } else {
                        trace -= 3;
                    }
                } else {
                    trace -= 2;
                }
            } else {
                trace -= 1;
            }
            return x;
        } finally {
            cleanup();
        }
    }
}
