<template>
  <div class="print-container p-4">
    <div class="mb-4 flex justify-between items-center no-print">
      <div>
        <label for="date-selector" class="mr-2">Select Date:</label>
        <input
          type="date"
          id="date-selector"
          v-model="selectedDate"
          class="border p-1 rounded"
        />
      </div>
      <div>
        <button
          @click="downloadPDF"
          :disabled="isGeneratingPdf"
          class="bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded mr-2 disabled:opacity-50"
        >
          {{ isGeneratingPdf ? "Generating..." : "Download PDF" }}
        </button>
        <button
          @click="printReport"
          class="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded no-print"
        >
          Print
        </button>
      </div>
    </div>

    <div id="pdf-render-area" class="pdf-render-area bg-white shadow-lg">
      <div class="page-content page-1">
        <div class="header common-header">
          <img
            src="/Gendejesus.png"
            alt="General de Jesus College Logo"
            class="header-logo"
          />
          <div class="header-text">
            <div class="college-name">General de Jesus College</div>
            <div class="address">
              VALLARTA ST. POBLACION, SAN ISIDRO, NUEVA ECIJA
            </div>
            <div class="contact">
              (+6344) 940-6161 | gdjcdejesus@gmail.com | gdjcdejesus.edu.ph
            </div>
          </div>
          <div class="report-title-line">
            Daily Record of Library Users SY: 2024-2025
            {{ formattedSelectedDateForTitle }}
          </div>
        </div>

        <div id="main-table-container">
          <table class="attendance-table">
            <thead>
              <tr>
                <th class="date-column column-name">DATE</th>
                <th class="time-column column-name">TIME</th>
                <th class="name-column column-name">
                  NAME (Last Name, First Name)
                </th>
                <th
                  v-for="course in courses"
                  :key="course"
                  class="course-column"
                >
                  <div class="vertical-text">{{ course }}</div>
                </th>
                <th class="purpose-column column-name">Purpose of Visit</th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="attendanceData.length === 0">
                <td :colspan="5 + courses.length" class="text-center py-4">
                  No attendance data for selected date.
                </td>
              </tr>
              <tr
                v-for="(entry, index) in attendanceData"
                :key="`row-${index}`"
              >
                <td>{{ entry.date }}</td>
                <td>{{ entry.time }}</td>
                <td class="text-left pl-1">{{ entry.name }}</td>
                <td
                  v-for="course in courses"
                  :key="`cell-${index}-${course}`"
                  class="course-cell"
                >
                  {{ entry.course === course ? "✓" : "" }}
                </td>
                <td class="text-left pl-1">{{ entry.purpose }}</td>
              </tr>
              <tr class="total-row">
                <td colspan="3" class="text-right pr-2">
                  <strong>Total:</strong>
                </td>
                <td
                  v-for="course in courses"
                  :key="`total-${course}`"
                  class="total-cell"
                >
                  <strong>{{ getTotalForCourse(course) }}</strong>
                </td>
                <td class="text-right pr-2">
                  <strong>Grand Total: {{ attendanceData.length }}</strong>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="footer print-only">
          Page <span class="page-number">1</span>
        </div>
      </div>

      <div class="page-content page-2 print-page-break">
        <div class="header common-header">
          <img
            src="/Gendejesus.png"
            alt="General de Jesus College Logo"
            class="header-logo"
          />
          <div class="header-text">
            <div class="college-name">General de Jesus College</div>
            <div class="address">
              VALLARTA ST. POBLACION, SAN ISIDRO, NUEVA ECIJA
            </div>
            <div class="contact">
              (+6344) 940-6161 | gdjcdejesus@gmail.com | gdjcdejesus.edu.ph
            </div>
          </div>
          <div class="report-title-line">
            Daily Record of Library Users SY: 2024-2025
            {{ formattedSelectedDateForTitle }}
          </div>
        </div>

        <div class="summary-section">
          <div class="summary-header full-width-header">
            Total Attendance by Purpose - {{ formattedSelectedDateForTitle }}
          </div>

          <div class="course-summary">
            <table class="course-summary-table">
              <tbody>
                <tr
                  v-for="(coursesChunk, chunkIndex) in chunkedCourses"
                  :key="`chunk-${chunkIndex}`"
                  class="course-summary-row"
                >
                  <template
                    v-for="(course, courseIndex) in coursesChunk"
                    :key="`summary-${course}`"
                  >
                    <td class="course-summary-label">
                      <strong>{{ course }}:</strong>
                    </td>
                    <td class="course-summary-value">
                      {{ getTotalForCourse(course) }}
                    </td>
                    <td
                      class="course-summary-spacer"
                      v-if="courseIndex < coursesChunk.length - 1"
                    ></td>
                  </template>
                  <template v-if="coursesChunk.length < 5">
                    <td
                      v-for="i in 5 - coursesChunk.length"
                      :key="`filler-${chunkIndex}-${i}`"
                      colspan="3"
                      class="course-summary-item empty"
                    ></td>
                  </template>
                </tr>
                <tr class="total-row course-total-row">
                  <td
                    colspan="15"
                    class="text-right pr-2 pt-2 border-t-2 border-black"
                  >
                    <strong
                      >Total Attendance: {{ attendanceData.length }}</strong
                    >
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="purpose-summary-container mt-4">
            <table class="purpose-summary-table">
              <thead>
                <tr>
                  <th
                    v-for="purpose in uniquePurposes"
                    :key="`purpose-header-${purpose}`"
                  >
                    {{ purpose.toUpperCase() }}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td
                    v-for="purpose in uniquePurposes"
                    :key="`purpose-count-${purpose}`"
                  >
                    {{ getTotalForPurpose(purpose) }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
        <div class="footer print-only">
          Page <span class="page-number">2</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, nextTick } from "vue";
import { jsPDF } from "jspdf";
import autoTable from "jspdf-autotable";
import { useToast } from "@/components/ui/toast/use-toast";
// If using an actual image, import it or use a public path
// import logoPath from '/Gendejesus.png'; // Example if using build tools

interface AttendanceEntry {
  date: string; // Keep original format for display if needed
  time: string;
  name: string;
  course: string;
  purpose: string;
}

export default defineComponent({
  name: "LibraryAttendanceRecord",
  setup() {
    const selectedDate = ref("2025-02-11"); // Match reference date example
    const { toast } = useToast();
    const isGeneratingPdf = ref(false);
    const logoBase64 = ref<string | null>(null); // To store logo image data

    // --- Constants for Header ---
    const collegeName = "General de Jesus College";
    const address = "VALLARTA ST. POBLACION, SAN ISIDRO, NUEVA ECIJA";
    const contactInfo =
      "(+6344) 940-6161 | gdjcdejesus@gmail.com | gdjcdejesus.edu.ph";
    const schoolYear = "2024-2025"; // Make dynamic if needed

    // --- Load Logo ---
    // Best practice: Load the logo once, convert to base64 for embedding in PDF
    const loadLogo = async () => {
      try {
        // IMPORTANT: Replace with the actual path to your logo
        const response = await fetch("/Gendejesus.png");
        if (!response.ok) throw new Error("Logo not found");
        const blob = await response.blob();
        const reader = new FileReader();
        reader.onloadend = () => {
          logoBase64.value = reader.result as string;
        };
        reader.readAsDataURL(blob);
      } catch (error) {
        console.error("Error loading logo:", error);
        toast({
          title: "Logo Error",
          description: "Could not load the college logo.",
          variant: "destructive",
        });
      }
    };
    // Call loadLogo when component mounts or before first PDF generation
    // onMounted(loadLogo); // Uncomment if using onMounted

    const courses = [
      "JHS",
      "ABM",
      "STEM",
      "HUMMS",
      "GAS",
      "BEEd",
      "BSEdEng",
      "BSEdSoc",
      "BSA",
      "BSAIS",
      "BSMA",
      "BSIA",
      "BSBA",
      "BSBAFM",
      "BSBAHRDM",
      "BSBAMM",
      "BSIT",
      "BSHM",
      "Faculty",
    ];

    // --- Sample Data (Add more to test pagination) ---
    const attendanceData = ref<AttendanceEntry[]>([
      // Add ~30-40 rows here to properly test page breaks
      {
        date: "Feb. 11, 2025",
        time: "08:05 am",
        name: "DOE, JOHN R.",
        course: "BSIT",
        purpose: "Research",
      },
      {
        date: "Feb. 11, 2025",
        time: "08:15 am",
        name: "SMITH, JANE M.",
        course: "BEEd",
        purpose: "Borrowing Books",
      },
      {
        date: "Feb. 11, 2025",
        time: "08:30 am",
        name: "CRUZ, MARIA S.",
        course: "STEM",
        purpose: "Reading/Study",
      },
      {
        date: "Feb. 11, 2025",
        time: "08:45 am",
        name: "GARCIA, LUIS P.",
        course: "Faculty",
        purpose: "Consultation",
      },
      {
        date: "Feb. 11, 2025",
        time: "09:00 am",
        name: "REYES, ANNA L.",
        course: "ABM",
        purpose: "Group Work",
      },
      {
        date: "Feb. 11, 2025",
        time: "09:10 am",
        name: "SANTOS, PETER T.",
        course: "BSHM",
        purpose: "Internet Access",
      },
      {
        date: "Feb. 11, 2025",
        time: "09:20 am",
        name: "RAMOS, ELENA G.",
        course: "JHS",
        purpose: "Reading/Study",
      },
      // ... Add many more entries (~30-40 total)
      {
        date: "Feb. 11, 2025",
        time: "11:00 am",
        name: "LEE, CHLOE K.",
        course: "GAS",
        purpose: "Research",
      },
      {
        date: "Feb. 11, 2025",
        time: "11:15 am",
        name: "WONG, DAVID Z.",
        course: "BSAIS",
        purpose: "Borrowing Books",
      },
      // Example last entry
      {
        date: "Feb. 11, 2025",
        time: "04:45 pm",
        name: "PEREZ, SOFIA B.",
        course: "BSEdEng",
        purpose: "Returning Books",
      },
    ]);

    // --- Computed Properties ---
    const formattedSelectedDateForTitle = computed(() => {
      const date = new Date(selectedDate.value + "T00:00:00"); // Ensure correct date parsing
      return date.toLocaleDateString("en-US", {
        month: "long",
        day: "numeric",
        year: "numeric",
      });
    });

    const formattedSelectedDate = computed(() => {
      const date = new Date(selectedDate.value + "T00:00:00");
      const options: Intl.DateTimeFormatOptions = {
        year: "numeric",
        month: "short",
        day: "2-digit",
      };
      return new Intl.DateTimeFormat("en-US", options).format(date);
    });

    const chunkedCourses = computed(() => {
      const chunkSize = 5; // 5 pairs of Label + Value per row
      const result = [];
      for (let i = 0; i < courses.length; i += chunkSize) {
        result.push(courses.slice(i, i + chunkSize));
      }
      return result;
    });

    const uniquePurposes = computed(() => {
      // Get unique purposes and sort them for consistency if needed
      const purposes = new Set(
        attendanceData.value.map((entry) => entry.purpose)
      );
      return Array.from(purposes).sort();
    });

    // --- Calculation Functions ---
    const getTotalForCourse = (course: string) => {
      return attendanceData.value.filter((entry) => entry.course === course)
        .length;
    };

    const getTotalForPurpose = (purpose: string) => {
      return attendanceData.value.filter((entry) => entry.purpose === purpose)
        .length;
    };

    // --- PDF Generation ---
    const downloadPDF = async () => {
      if (isGeneratingPdf.value) return;
      isGeneratingPdf.value = true;
      await loadLogo(); // Ensure logo is loaded

      if (!logoBase64.value) {
        toast({
          title: "Error",
          description: "Logo image not loaded. Cannot generate PDF.",
          variant: "destructive",
        });
        isGeneratingPdf.value = false;
        return;
      }

      toast({ title: "Generating PDF", description: "Please wait..." });

      try {
        // Ff-Long (13 inches), Letter width (8.5 inches), Landscape
        const pdf = new jsPDF({
          orientation: "landscape",
          unit: "in",
          format: [13, 8.5], // Folio (Long Bond) approx 13x8.5 inches
        });

        const pageHeight = pdf.internal.pageSize.getHeight();
        const pageWidth = pdf.internal.pageSize.getWidth();
        const margin = 0.5; // inches

        const addHeader = (pdfInstance: jsPDF) => {
          const logoWidth = 0.6;
          const logoHeight = 0.6;
          const logoX = margin;
          const logoY = margin / 2;
          const textStartX = logoX + logoWidth + 0.1;

          // Draw Logo
          if (logoBase64.value) {
            pdfInstance.addImage(
              logoBase64.value,
              "PNG",
              logoX,
              logoY,
              logoWidth,
              logoHeight
            );
          }

          // Draw Header Text
          pdfInstance.setFontSize(14).setFont(undefined, "bold");
          pdfInstance.text(collegeName, textStartX, logoY + 0.15); // Adjust Y pos

          pdfInstance.setFontSize(8).setFont(undefined, "normal");
          pdfInstance.text(address, textStartX, logoY + 0.3);
          pdfInstance.text(contactInfo, textStartX, logoY + 0.45);

          // Report Title Line (below header text, centered maybe?)
          pdfInstance.setFontSize(10).setFont(undefined, "bold");
          const reportTitle = `Daily Record of Library Users SY: ${schoolYear} ${formattedSelectedDateForTitle.value}`;
          const titleWidth = pdfInstance.getTextWidth(reportTitle);
          pdfInstance.text(
            reportTitle,
            (pageWidth - titleWidth) / 2,
            logoY + logoHeight + 0.15
          ); // Below logo/text block
        };

        const addFooter = (
          pdfInstance: jsPDF,
          pageNumber: number,
          totalPages: number
        ) => {
          pdfInstance.setFontSize(8).setFont(undefined, "normal");
          const footerText = `Page ${pageNumber} of ${totalPages}`;
          const textWidth = pdfInstance.getTextWidth(footerText);
          pdfInstance.text(
            footerText,
            pageWidth - margin - textWidth,
            pageHeight - margin / 2
          );
        };

        // --- Main Attendance Table ---
        const head = [
          // Column headers - need to match the table data structure
          {
            content: "DATE",
            styles: { halign: "center", fontStyle: "bold", fontSize: 7 },
          },
          {
            content: "TIME",
            styles: { halign: "center", fontStyle: "bold", fontSize: 7 },
          },
          {
            content: "NAME (Last Name, First Name)",
            styles: { halign: "left", fontStyle: "bold", fontSize: 7 },
          },
          // Dynamically add course headers
          ...courses.map((course) => ({
            content: course, // Use jsPDF-Autotable's vertical text handling if needed, or pre-rotate
            styles: {
              halign: "center",
              fontStyle: "bold",
              fontSize: 6,
              cellWidth: 0.3,
            }, // Adjust width
          })),
          {
            content: "Purpose of Visit",
            styles: { halign: "left", fontStyle: "bold", fontSize: 7 },
          },
        ];

        const body = attendanceData.value.map((entry) => [
          entry.date,
          entry.time,
          { content: entry.name, styles: { halign: "left", fontSize: 7 } },
          ...courses.map((course) => (entry.course === course ? "✓" : "")),
          { content: entry.purpose, styles: { halign: "left", fontSize: 7 } },
        ]);

        // Add the 'Total' row data
        const totalRow = [
          {
            content: "Total:",
            colSpan: 3,
            styles: { halign: "right", fontStyle: "bold", fontSize: 7 },
          },
          ...courses.map((course) => ({
            content: getTotalForCourse(course).toString(),
            styles: { halign: "center", fontStyle: "bold", fontSize: 7 },
          })),
          {
            content: `Grand Total: ${attendanceData.value.length}`,
            styles: { halign: "right", fontStyle: "bold", fontSize: 7 },
          },
        ];
        body.push(totalRow);

        let finalY = 0; // Keep track of the vertical position
        let currentPage = 0; // Track current page number

        autoTable(pdf, {
          head: [head], // Wrap head in an array
          body: body,
          startY: 1.5, // Start table below header (adjust as needed)
          margin: { top: 1.5, right: margin, bottom: margin, left: margin },
          theme: "grid", // 'striped', 'grid', 'plain'
          styles: {
            // Global styles
            fontSize: 7,
            cellPadding: 0.02, // inches
            overflow: "linebreak", // Prevent text overflow issues
          },
          headStyles: {
            fillColor: [230, 230, 230], // Light gray background for header
            textColor: [0, 0, 0],
            fontStyle: "bold",
            lineWidth: 0.01,
            lineColor: [0, 0, 0],
          },
          bodyStyles: {
            lineWidth: 0.01,
            lineColor: [0, 0, 0],
          },
          alternateRowStyles: {
            // fillColor: [245, 245, 245] // Optional: zebra striping
          },
          columnStyles: {
            // Example: Set specific width for Name column
            2: { cellWidth: 2.5, halign: "left" }, // Name column (index 2)
            // Course columns (adjust index range based on Date, Time, Name)
            // Indices 3 to 3 + courses.length - 1
            ...Object.fromEntries(
              courses.map((_, i) => [
                3 + i,
                { cellWidth: 0.3, halign: "center", fontSize: 8 },
              ])
            ), // Checkmark columns
            // Purpose column (last one)
            [3 + courses.length]: { cellWidth: "auto", halign: "left" }, // Purpose column
          },
          didParseCell: (data) => {
            // Center checkmarks vertically if needed (might not be necessary)
            if (
              courses.includes(data.column.dataKey as string) &&
              data.cell.raw === "✓"
            ) {
              // data.cell.styles.valign = 'middle'; // Already default?
            }
          },
          didDrawPage: (data) => {
            currentPage = data.pageNumber;
            addHeader(pdf);
            // Footer added after knowing total pages
            finalY = data.cursor?.y ?? pageHeight - margin; // Update position after table draw
          },
          // Add a hook for the last row (Total row) styling if needed
          willDrawCell: (data) => {
            if (data.row.index === body.length - 1) {
              // Is it the total row?
              data.cell.styles.fontStyle = "bold";
              data.cell.styles.fillColor = [240, 240, 240]; // Slightly different background?
            }
          },
        });

        // --- Determine if Summary goes on new page ---
        const summaryStartY = finalY + 0.3; // Add some space
        const summarySectionHeightEstimate = 2.5; // Estimate inches needed for summary (adjust!)

        if (
          currentPage === 1 &&
          summaryStartY + summarySectionHeightEstimate > pageHeight - margin
        ) {
          pdf.addPage();
          currentPage++;
          addHeader(pdf); // Add header to the new page
          finalY = 1.5; // Reset Y for the new page (below header)
        } else {
          finalY = summaryStartY; // Continue on the current page
        }

        // --- Add Summary Sections ---
        const summaryStartX = margin;
        const summaryWidth = pageWidth - 2 * margin;

        // Summary Header
        pdf.setFontSize(12).setFont(undefined, "bold");
        pdf.setFillColor(204, 204, 204); // Gray background
        pdf.rect(summaryStartX, finalY, summaryWidth, 0.3, "F"); // Draw background rect
        pdf.setTextColor(0, 0, 0);
        const summaryHeaderText = `Total Attendance by Purpose - ${formattedSelectedDateForTitle.value}`;
        const summaryHeaderWidth = pdf.getTextWidth(summaryHeaderText);
        pdf.text(
          summaryHeaderText,
          (pageWidth - summaryHeaderWidth) / 2,
          finalY + 0.2
        ); // Centered text
        finalY += 0.4; // Move down past the header

        // Course Summary Table (using autoTable)
        const courseSummaryBody = [];
        for (const chunk of chunkedCourses.value) {
          const row: any[] = [];
          chunk.forEach((course) => {
            row.push(
              {
                content: `${course}:`,
                styles: { fontStyle: "bold", halign: "left" },
              },
              {
                content: getTotalForCourse(course).toString(),
                styles: { halign: "left" },
              }
            );
          });
          // Fill empty slots if chunk has fewer than 5 courses to maintain structure
          // Each course takes 2 cells (label + value)
          const cellsInRow = row.length;
          const targetCells = 10; // 5 courses * 2 cells each
          if (cellsInRow < targetCells) {
            row.push({
              content: "",
              colSpan: targetCells - cellsInRow,
              styles: { fillColor: [255, 255, 255] },
            }); // Invisible filler
          }
          courseSummaryBody.push(row);
        }
        // Add Total Attendance row
        courseSummaryBody.push([
          {
            content: `Total Attendance: ${attendanceData.value.length}`,
            colSpan: 10, // Span all columns
            styles: { halign: "right", fontStyle: "bold", fontSize: 9 },
          },
        ]);

        autoTable(pdf, {
          body: courseSummaryBody,
          startY: finalY,
          theme: "plain", // No grid for this one, like the reference
          margin: { left: margin, right: margin },
          tableWidth: summaryWidth,
          styles: { fontSize: 8, cellPadding: 0.05 },
          columnStyles: {
            // Make label columns slightly wider maybe
            // 0, 2, 4, 6, 8 are labels; 1, 3, 5, 7, 9 are values
            0: { cellWidth: 0.8 },
            1: { cellWidth: 0.5 },
            2: { cellWidth: 0.8 },
            3: { cellWidth: 0.5 },
            4: { cellWidth: 0.8 },
            5: { cellWidth: 0.5 },
            6: { cellWidth: 0.8 },
            7: { cellWidth: 0.5 },
            8: { cellWidth: 0.8 },
            9: { cellWidth: 0.5 },
          },
          willDrawCell: (data) => {
            // Add top border for the Total Attendance row
            if (data.row.index === courseSummaryBody.length - 1) {
              pdf.setDrawColor(0, 0, 0); // Black
              pdf.setLineWidth(0.015); // Thicker line
              pdf.line(
                data.cell.x,
                data.cell.y,
                data.cell.x + data.cell.width,
                data.cell.y
              );
            }
          },
          didDrawPage: (data) => {
            // In case summary spans pages
            addHeader(pdf);
            currentPage = data.pageNumber;
            finalY = data.cursor?.y ?? pageHeight - margin;
          },
          didDrawTable: (data) => {
            finalY = data.cursor.y + 0.1; // Move below course summary
          },
        });

        // Purpose Summary Table (using autoTable)
        const purposeHead = [
          uniquePurposes.value.map((p) => ({
            content: p.toUpperCase(), // Match reference style
            styles: { halign: "center", fontStyle: "bold", fontSize: 8 },
          })),
        ];
        const purposeBody = [
          uniquePurposes.value.map((p) => ({
            content: getTotalForPurpose(p).toString(),
            styles: { halign: "center", fontSize: 8 },
          })),
        ];

        autoTable(pdf, {
          head: purposeHead,
          body: purposeBody,
          startY: finalY + 0.2, // Space before purpose table
          theme: "grid",
          margin: { left: margin * 4, right: margin * 4 }, // Indent this table more
          styles: { fontSize: 8, cellPadding: 0.05 },
          headStyles: {
            fillColor: [230, 230, 230],
            textColor: [0, 0, 0],
            fontStyle: "bold",
          },
          didDrawPage: (data) => {
            // In case summary spans pages
            addHeader(pdf);
            currentPage = data.pageNumber;
            // Footer added after knowing total pages
          },
        });

        // --- Add Footers to all pages ---
        const totalPages = (pdf as any).internal.getNumberOfPages();
        for (let i = 1; i <= totalPages; i++) {
          pdf.setPage(i);
          addFooter(pdf, i, totalPages);
        }

        // --- Save PDF ---
        pdf.save(`library-attendance-${selectedDate.value}.pdf`);

        toast({
          title: "PDF Generated Successfully",
          description: `library-attendance-${selectedDate.value}.pdf downloaded.`,
          variant: "success",
        });
      } catch (error) {
        console.error("Error generating PDF:", error);
        toast({
          title: "PDF Generation Failed",
          description:
            error instanceof Error
              ? error.message
              : "An unknown error occurred",
          variant: "destructive",
        });
      } finally {
        isGeneratingPdf.value = false;
      }
    };

    // --- Print Function ---
    const printReport = async () => {
      // Ensure styles for print are applied before triggering print
      toast({ title: "Preparing Print", description: "Please wait..." });
      await nextTick(); // Wait for DOM updates if any
      window.print();
      // Note: Cannot easily add dynamic headers/footers here like in PDF
      toast({
        title: "Print Dialog Opened",
        description:
          "Adjust print settings as needed (Landscape, Scale: Fit to printable area).",
        variant: "success",
      });
    };

    return {
      selectedDate,
      formattedSelectedDate,
      formattedSelectedDateForTitle,
      courses,
      attendanceData,
      chunkedCourses,
      uniquePurposes,
      getTotalForCourse,
      getTotalForPurpose,
      downloadPDF,
      printReport,
      isGeneratingPdf,
      // Expose header info if needed in template directly
      collegeName,
      address,
      contactInfo,
      schoolYear,
    };
  },
});
</script>

<style scoped>
/* Base styles */
.print-container {
  width: 100%;
  max-width: 1300px; /* Approx landscape long bond paper width */
  margin: 0 auto;
  font-family: Arial, sans-serif; /* Match typical PDF fonts */
}

.pdf-render-area {
  border: 1px solid #ccc;
  margin-top: 1rem;
  padding: 0.5in; /* Simulate PDF margins for preview */
  /* Mimic paper size for preview - aspect ratio for 13x8.5 */
  /* width: 100%; */
  /* aspect-ratio: 13 / 8.5; */
  overflow: hidden; /* Hide anything overflowing the 'paper' */
}

/* Header Styles */
.common-header {
  display: flex;
  align-items: center; /* Vertically align logo and text */
  margin-bottom: 10px; /* Space below header */
  padding-bottom: 10px;
  /* border-bottom: 1px solid black; */ /* Optional separator */
}

.header-logo {
  width: 60px; /* Adjust size as needed */
  height: 60px;
  margin-right: 15px;
  object-fit: contain;
}

.header-text {
  flex-grow: 1;
  text-align: left;
}

.college-name {
  font-size: 16px; /* Slightly larger */
  font-weight: bold;
  font-family: "Times New Roman", Times, serif; /* Serif font like reference */
  margin-bottom: 2px;
}

.address,
.contact {
  font-size: 9px;
  line-height: 1.2;
}

.report-title-line {
  width: 100%;
  text-align: center;
  font-size: 11px;
  font-weight: bold;
  margin-top: 10px; /* Space above title line */
  position: absolute; /* Position relative to pdf-render-area */
  top: 85px; /* Adjust based on logo/header text height */
  left: 0;
}

/* Table Styles */
table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
  margin-bottom: 15px;
  font-size: 8px; /* Base font size for table content */
}

th,
td {
  border: 1px solid black;
  padding: 1px 3px; /* Minimal padding */
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap; /* Prevent wrapping by default */
  vertical-align: middle; /* Center vertically */
}

th.column-name {
  font-weight: bold;
  font-size: 9px; /* Slightly larger header text */
  background-color: #e6e6e6; /* Light gray header */
  padding: 4px 3px;
}

.attendance-table .text-left {
  text-align: left;
}
.attendance-table .text-right {
  text-align: right;
}

.vertical-text {
  writing-mode: vertical-rl;
  text-orientation: mixed; /* Or upright */
  white-space: nowrap;
  transform: rotate(180deg);
  font-size: 8px; /* Small font for vertical */
  line-height: 1;
  padding: 5px 0px; /* Adjust padding */
  display: inline-block; /* Necessary for vertical alignment */
  margin: 0 auto; /* Center within the cell */
  max-height: 50px; /* Limit height */
}

/* Column Widths (adjust percentages based on content and total columns) */
.date-column {
  width: 7%;
}
.time-column {
  width: 6%;
}
.name-column {
  width: 18%;
}
.course-column {
  width: 3%;
  max-width: 25px;
  padding: 0;
} /* Fixed width for course cols */
.purpose-column {
  width: 15%;
}

.course-cell {
  font-size: 10px; /* Make checkmark slightly larger */
  font-weight: bold;
}

/* Total Row */
.total-row td {
  font-weight: bold;
  background-color: #f2f2f2;
  font-size: 8px;
  padding: 3px;
}
.total-cell {
  font-weight: bold;
  font-size: 9px;
}

/* Summary Section Styles */
.summary-section {
  margin-top: 20px; /* Space above summary */
  /* page-break-before: always; */ /* Force page break before this in print */
}

.full-width-header {
  width: 100%;
  background-color: #cccccc; /* Gray background */
  padding: 8px;
  text-align: center;
  color: black;
  font-size: 12px;
  font-weight: bold;
  margin: 15px 0;
  box-sizing: border-box;
}

/* Course Summary Table */
.course-summary {
  width: 100%;
  margin-bottom: 20px;
}
.course-summary-table {
  border-collapse: separate; /* Use separate for spacing */
  border-spacing: 0 2px; /* Add vertical space between rows */
  font-size: 9px;
  table-layout: auto; /* Allow columns to size based on content */
}
.course-summary-table td {
  border: none; /* No borders within the summary table cells */
  padding: 1px 5px;
  white-space: nowrap;
  text-align: left;
  vertical-align: top;
}

.course-summary-label {
  font-weight: bold;
  width: auto; /* Adjust as needed */
}
.course-summary-value {
  width: 30px; /* Fixed width for value */
  text-align: left;
}
.course-summary-spacer {
  width: 20px; /* Space between course pairs */
}

.course-total-row td {
  border-top: 2px solid black !important; /* Thick top border */
  padding-top: 5px !important;
  font-size: 10px;
}

/* Purpose Summary Table */
.purpose-summary-container {
  width: 70%; /* Make it less wide than the page */
  margin: 20px auto 0 auto; /* Center it */
}
.purpose-summary-table th,
.purpose-summary-table td {
  border: 1px solid black;
  padding: 4px;
  text-align: center;
  font-size: 9px;
}
.purpose-summary-table th {
  font-weight: bold;
  background-color: #e6e6e6;
}

/* Footer Styles */
.footer {
  text-align: right;
  font-size: 8px;
  margin-top: 15px;
  width: 100%;
  position: absolute; /* Position relative to page-content */
  bottom: 0.3in; /* Position near the bottom margin */
  right: 0.5in;
}

/* --- Print Specific Styles --- */
@media print {
  body {
    margin: 0;
    padding: 0;
    font-size: 8pt; /* Base font size for print */
    -webkit-print-color-adjust: exact !important; /* Force background colors (like headers) in Chrome/Safari */
    print-color-adjust: exact !important; /* Standard */
  }

  .no-print {
    display: none !important; /* Hide buttons, date selector */
  }

  .print-container {
    max-width: none; /* Remove max-width */
    margin: 0;
    padding: 0;
  }

  .pdf-render-area {
    border: none !important;
    box-shadow: none !important;
    margin: 0 !important;
    padding: 0 !important; /* Remove padding used for screen preview */
  }

  .page-content {
    page-break-inside: avoid; /* Try to keep content of a page together */
    padding: 0.5in; /* Add back margins for printing */
    position: relative; /* Needed for absolute footer positioning */
    height: 7.5in; /* Approx printable height on 8.5in landscape */
    width: 12in; /* Approx printable width on 13in landscape */
    box-sizing: border-box;
  }

  .print-page-break {
    page-break-before: always !important; /* Force break before summary */
  }

  .common-header {
    /* Headers should repeat via PDF generation, hide duplicate HTML headers in print */
    /* display: none; */ /* Or style them if needed */
  }

  /* Ensure table headers repeat on each page if table breaks (browser dependent) */
  thead {
    display: table-header-group;
  }
  /* Ensure total row stays with table */
  tbody tr.total-row {
    page-break-inside: avoid;
  }

  /* Summary section page break */
  .summary-section {
    margin-top: 0; /* Remove extra margin potentially added by page break */
  }

  .footer.print-only {
    display: block; /* Show the simple footer in print */
    position: fixed; /* Try fixed positioning for print footer */
    bottom: 10px;
    right: 20px;
  }

  /* Ensure vertical text prints correctly */
  .vertical-text {
    /* May need adjustments based on browser */
  }
}

/* Page Setup for Print */
@page {
  size: 13in 8.5in landscape; /* Long bond landscape */
  margin: 0.5in; /* Standard margins */
}
</style>
