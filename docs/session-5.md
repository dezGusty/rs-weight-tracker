# 2023.02.25 ChatGPT session 5

## Input 1

In the code you provided, when the `Submit` button is clicked the first time, the chart is created correctly, but at the second click on the `Submit` button I get an error stating that:

"Chart with ID '0' must be destroyed before the canvas with ID 'chart' can be reused."

Could you please destroy the chart item if it was already created ?

Yes, we can check if a chart already exists before creating a new chart instance and destroy the previous one. Here's the modified script:

```html

<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8">
    <title>Rolling Average</title>
  </head>
  <body>
    <form>
      <label for="start-date">Start Date:</label>
      <input type="date" id="start-date" name="start-date"><br>

      <label for="end-date">End Date:</label>
      <input type="date" id="end-date" name="end-date"><br>

      <label for="days">Days:</label>
      <input type="number" id="days" name="days" min="1" max="7" value="7"><br>

      <button type="submit" id="submit">Submit</button>
    </form>

    <div>
      <canvas id="chart"></canvas>
    </div>

    <table id="result-table">
      <thead>
        <tr>
          <th>Date</th>
          <th>Weight</th>
        </tr>
      </thead>
      <tbody></tbody>
    </table>

    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script>
      const form = document.querySelector('form');
      const tableBody = document.querySelector('table tbody');
      let chart;

      form.addEventListener('submit', (event) => {
        event.preventDefault();

        const startDate = document.getElementById('start-date').value;
        const endDate = document.getElementById('end-date').value;
        const days = document.getElementById('days').value;

        fetch(`/api/rolling_average?start_date=${startDate}&end_date=${endDate}&days=${days}`)
          .then(response => response.json())
          .then(data => {
            tableBody.innerHTML = '';
            data.forEach(entry => {
              const row = document.createElement('tr');
              const dateCell = document.createElement('td');
              const weightCell = document.createElement('td');

              dateCell.textContent = entry.date;
              weightCell.textContent = `${entry.average.toFixed(1)} kg`;

              row.appendChild(dateCell);
              row.appendChild(weightCell);

              tableBody.appendChild(row);
            });

            if (chart) {
              chart.destroy();
            }

            const chartCanvas = document.getElementById('chart');
            chart = new Chart(chartCanvas, {
              type: 'line',
              data: {
                labels: data.map(entry => entry.date),
                datasets: [
                  {
                    label: 'Rolling Average',
                    data: data.map(entry => entry.average.toFixed(1)),
                    fill: false,
                    borderColor: 'rgb(75, 192, 192)',
                    tension: 0.1,
                  },
                ],
              },
              options: {
                scales: {
                  y: {
                    beginAtZero: true,
                    suggestedMax: 100,
                  },
                },
              },
            });
          })
          .catch(error => {
            console.error(error);
          });
      });
    </script>
  </body>
</html>
```

This should destroy the previous chart instance before creating a new one.