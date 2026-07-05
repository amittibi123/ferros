pub struct Process {
    pid: u64,
    page_table: PhysAddr,   // CR3 שלו
    kernel_stack: VirtAddr,
    context: CpuContext,    // רגיסטרים שמורים
    state: ProcessState,    // Running/Ready/Blocked
}