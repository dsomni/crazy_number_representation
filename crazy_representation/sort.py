inputF = open("output.txt",'r')
lines = inputF.readlines()
inputF.close()
outputF = open("output.txt",'w')
lines.sort(key=lambda x: int(x.split('=')[0]))
outputF.writelines(lines)