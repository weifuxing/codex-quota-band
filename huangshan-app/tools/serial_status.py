"""Read only the application's bounded status line, never retain UART logs."""
import argparse,re,time
import serial
parser=argparse.ArgumentParser()
parser.add_argument('--port',default='COM4')
parser.add_argument('--baud',type=int,default=1000000)
parser.add_argument('--threads',action='store_true')
args=parser.parse_args()
pattern=re.compile(r'CQ_STATUS state=[0-3] five=(?:[0-9]{1,3}) weekly=(?:[0-9]{1,3}) packets=[0-9]{1,10} confirmed=[0-9]{1,10}(?: ble=[0-9]{1,2} error=[0-9]{1,3})?(?: layout=[01] clicks=[0-9]{1,10})?(?: screen=[01] brightness=(?:0|30) shift=-?[0-2],-?[0-2])?')
uart=serial.Serial(baudrate=args.baud,timeout=0.2)
uart.dtr=False
uart.rts=False
uart.port=args.port
with uart:
    uart.reset_input_buffer()
    uart.write(b'\r\nlist_thread\r\n' if args.threads else b'\r\ncq_status\r\n')
    end=time.monotonic()+3
    found=False
    screen_pattern=re.compile(r'CQ_SCREEN on=[01] brightness=(?:0|30) shift=-?[0-2],-?[0-2] idle=[0-9]{1,10}')
    while time.monotonic()<end:
        line=uart.readline(512).decode('utf-8',errors='replace')
        if args.threads:
            thread=re.search(r'^([A-Za-z0-9_]{1,8})\s+[0-9]+\s+(ready|running|suspend|init|close)',line.strip())
            if thread:
                print(f'THREAD {thread.group(1)} {thread.group(2)}');found=True
            continue
        match=pattern.search(line)
        if match:
            print(match.group(0));found=True
        screen=screen_pattern.search(line)
        if screen:
            print(screen.group(0));break
    if not found:
        print('No application status response; check boot, baud rate and console configuration.')
        raise SystemExit(1)
