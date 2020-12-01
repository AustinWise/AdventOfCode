using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace day1
{
    class Program
    {
        static void Main(string[] args)
        {
            var ints = File.ReadAllLines("input.txt").Select(l => int.Parse(l)).ToArray();
            Array.Sort(ints);
            int lo = 0;
            int hi = ints.Length - 1;
            while (true)
            {
                int sum = ints[lo] + ints[hi];
                if (sum < 2020)
                    lo++;
                else if (sum > 2020)
                    hi--;
                else
                {
                    Console.WriteLine(ints[lo] * ints[hi]);
                    break;
                }

                if (lo > hi)
                    throw new Exception("Did not find");
            }

            var intSet = new HashSet<int>(ints);

            for (int i = 0; i < ints.Length; i++)
            {
                for (int j = i + 1; j < ints.Length; j++)
                {
                    int x = ints[i];
                    int y = ints[j];
                    int z = 2020 - x - y;
                    if (intSet.Contains(z))
                    {
                        Console.WriteLine(x * y * z);
                        return;
                    }
                }
            }
        }
    }
}
