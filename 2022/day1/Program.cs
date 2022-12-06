using System.Collections.Generic;
using System.Diagnostics;

var elfCalories = new PriorityQueue<int, int>();

int cur = 0;
foreach (var line in day1.Properties.Resources.Input.Split('\n').Select(l => l.Trim('\r')))
{
    if (line.Length == 0)
    {
        elfCalories.Enqueue(cur, -1 * cur);
        cur = 0;
    }
    else
    {
        cur += int.Parse(line);
    }
}

int max = elfCalories.Dequeue();
Debug.Assert(max == 75622);

Console.WriteLine($"part one answer: {max}");

int n2 = elfCalories.Dequeue();
int n3 = elfCalories.Dequeue();

Console.WriteLine($"part two answer: {max + n2 + n3}");
