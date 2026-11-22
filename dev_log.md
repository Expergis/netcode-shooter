# 14/11/2026
Finally had the time to pick up this project again (my uni is stealing my soul), I also worked a little bit on this project the other day but I forgot to add it to the devlog. 

The other day I worked on the sequence.rs file, most of the time was spent trying to understand what the wrapping_sub function does. Also learned that variables that start with '_' supress unused variable compiler warning

Today I'm working on ring_buffer.rs, this struct is supposed to be a fixed size circular buffer that does not use heap memory. **Why avoid heap memory?** When allocating heap memory the kernel needs to look for free memory blocks and manage internal fragmentation, this introduces unpredicatble latency also known as **jitter**. 

Challenge of the day: understanding *MaybeUninit* and *unsafe* keyword

# 22/11/2026

Slow and steady wins the race. Only worked on the state rewind logic today, understanding **rewind_and_query** function declaration was harder than the logic. 

Learned about **traits** and **generic parameter bounds**.