using System.Diagnostics;

var leftList = new List<int>();
var rightList = new List<int>();

foreach (var line in File.ReadAllLines("input.txt"))
{
    if (line.Length == 0)
        continue;
    string[] numberStrs = line.Split(' ', StringSplitOptions.RemoveEmptyEntries);
    if (numberStrs.Length != 2)
        throw new Exception("unpexted length");
    leftList.Add(int.Parse(numberStrs[0]));
    rightList.Add(int.Parse(numberStrs[1]));
}

Debug.Assert(leftList.Count == rightList.Count);

leftList.Sort();
rightList.Sort();

int sumOfDistances = 0;
for (int i = 0; i < leftList.Count; i++)
{
    sumOfDistances += Math.Abs(leftList[i] - rightList[i]);
}

Console.WriteLine(sumOfDistances);